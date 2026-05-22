#![no_std]
#![no_main]

mod keymap;

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::interrupt::{self, InterruptExt};
use embassy_nrf::mode::Async;
use embassy_nrf::peripherals::{RNG, USBD};
use embassy_nrf::saadc::{self, Input as _, Saadc};
use embassy_nrf::usb::Driver;
use embassy_nrf::usb::vbus_detect::HardwareVbusDetect;
use embassy_nrf::{bind_interrupts, rng, usb};
use nrf_mpsl::Flash;
use nrf_sdc::mpsl::MultiprotocolServiceLayer;
use nrf_sdc::{self as sdc, mpsl};
use panic_probe as _;
use rand_chacha::ChaCha12Rng;
use rand_core::SeedableRng;
use rmk::ble::build_ble_stack;
use rmk::config::{
    BehaviorConfig, BleBatteryConfig, PositionalConfig, RmkConfig, StorageConfig,
};
use rmk::debounce::default_debouncer::DefaultDebouncer;
use rmk::futures::future::join;
use rmk::input_device::adc::{AnalogEventType, NrfAdc};
use rmk::input_device::battery::BatteryProcessor;
use rmk::input_device::joystick::JoystickProcessor;
use rmk::input_device::Runnable;
use rmk::split::ble::central::scan_peripherals;
use rmk::split::central::run_peripheral_manager;
use rmk::keyboard::Keyboard;
use rmk::{HostResources, initialize_encoder_keymap_and_storage};
use static_cell::StaticCell;

use keymap::{
    get_default_encoder_map, get_default_keymap, KEYBOARD_DEVICE_CONFIG, NUM_ENCODER, NUM_LAYER,
    ROW, COL, VIAL_CONFIG,
};

bind_interrupts!(struct Irqs {
    USBD => usb::InterruptHandler<USBD>;
    SAADC => saadc::InterruptHandler;
    RNG => rng::InterruptHandler<RNG>;
    EGU0_SWI0 => nrf_sdc::mpsl::LowPrioInterruptHandler;
    CLOCK_POWER => nrf_sdc::mpsl::ClockInterruptHandler, usb::vbus_detect::InterruptHandler;
    RADIO => nrf_sdc::mpsl::HighPrioInterruptHandler;
    TIMER0 => nrf_sdc::mpsl::HighPrioInterruptHandler;
    RTC0 => nrf_sdc::mpsl::HighPrioInterruptHandler;
});

#[embassy_executor::task]
async fn mpsl_task(mpsl: &'static MultiprotocolServiceLayer<'static>) -> ! {
    mpsl.run().await
}

/// How many outgoing L2CAP buffers per link
const L2CAP_TXQ: u8 = 3;

/// How many incoming L2CAP buffers per link
const L2CAP_RXQ: u8 = 3;

/// Size of L2CAP packets
const L2CAP_MTU: usize = 251;

fn build_sdc<'d, const N: usize>(
    p: nrf_sdc::Peripherals<'d>,
    rng: &'d mut rng::Rng<Async>,
    mpsl: &'d MultiprotocolServiceLayer,
    mem: &'d mut sdc::Mem<N>,
) -> Result<nrf_sdc::SoftdeviceController<'d>, nrf_sdc::Error> {
    sdc::Builder::new()?
        .support_scan()?
        .support_central()?
        .support_adv()?
        .support_peripheral()?
        .support_dle_peripheral()?
        .support_dle_central()?
        .support_phy_update_central()?
        .support_phy_update_peripheral()?
        .support_le_2m_phy()?
        .central_count(2)?
        .peripheral_count(1)?
        .buffer_cfg(L2CAP_MTU as u16, L2CAP_MTU as u16, L2CAP_TXQ, L2CAP_RXQ)?
        .build(p, rng, mpsl, mem)
}

fn ble_addr() -> [u8; 6] {
    let ficr = embassy_nrf::pac::FICR;
    let high = u64::from(ficr.deviceid(1).read());
    let addr = high << 32 | u64::from(ficr.deviceid(0).read());
    let addr = addr | 0x0000_c000_0000_0000;
    addr.to_le_bytes()[..6].try_into().expect("FICR read failed")
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("RoKi central starting");

    // nRF config
    let mut nrf_config = embassy_nrf::config::Config::default();
    nrf_config.dcdc.reg0_voltage = Some(embassy_nrf::config::Reg0Voltage::_3V3);
    nrf_config.dcdc.reg0 = false;
    nrf_config.dcdc.reg1 = false;
    let p = embassy_nrf::init(nrf_config);

    // MPSL
    let mpsl_p = mpsl::Peripherals::new(
        p.RTC0, p.TIMER0, p.TEMP, p.PPI_CH19, p.PPI_CH30, p.PPI_CH31,
    );
    let lfclk_cfg = mpsl::raw::mpsl_clock_lfclk_cfg_t {
        source: mpsl::raw::MPSL_CLOCK_LF_SRC_RC as u8,
        rc_ctiv: mpsl::raw::MPSL_RECOMMENDED_RC_CTIV as u8,
        rc_temp_ctiv: mpsl::raw::MPSL_RECOMMENDED_RC_TEMP_CTIV as u8,
        accuracy_ppm: mpsl::raw::MPSL_DEFAULT_CLOCK_ACCURACY_PPM as u16,
        skip_wait_lfclk_started: mpsl::raw::MPSL_DEFAULT_SKIP_WAIT_LFCLK_STARTED != 0,
    };
    static MPSL: StaticCell<MultiprotocolServiceLayer> = StaticCell::new();
    static SESSION_MEM: StaticCell<mpsl::SessionMem<1>> = StaticCell::new();
    let mpsl = MPSL.init(
        mpsl::MultiprotocolServiceLayer::with_timeslots(
            mpsl_p,
            Irqs,
            lfclk_cfg,
            SESSION_MEM.init(mpsl::SessionMem::new()),
        )
        .unwrap(),
    );
    spawner.must_spawn(mpsl_task(&*mpsl));

    // SDC / BLE controller
    let sdc_p = sdc::Peripherals::new(
        p.PPI_CH17, p.PPI_CH18, p.PPI_CH20, p.PPI_CH21, p.PPI_CH22, p.PPI_CH23,
        p.PPI_CH24, p.PPI_CH25, p.PPI_CH26, p.PPI_CH27, p.PPI_CH28, p.PPI_CH29,
    );
    let mut rng = rng::Rng::new(p.RNG, Irqs);
    let mut rng_gen = ChaCha12Rng::from_rng(&mut rng).unwrap();
    let mut sdc_mem = sdc::Mem::<8704>::new();
    let sdc = build_sdc(sdc_p, &mut rng, mpsl, &mut sdc_mem).unwrap();
    let mut host_resources = HostResources::new();
    let stack = build_ble_stack(sdc, ble_addr(), &mut rng_gen, &mut host_resources).await;

    // USB driver
    let driver = Driver::new(p.USBD, Irqs, HardwareVbusDetect::new(Irqs));

    // Behavior config
    let mut behavior_config = BehaviorConfig::default();

    // Flash + storage config
    let storage_config = StorageConfig {
        num_sectors: 32,
        start_addr: 0xA0000,
        clear_storage: false,
        clear_layout: false,
    };
    let flash = Flash::take(mpsl, p.NVMC);

    // RMK config
    let ble_battery_config = BleBatteryConfig::new(None, false, None, false);
    let rmk_config = RmkConfig {
        device_config: KEYBOARD_DEVICE_CONFIG,
        vial_config: VIAL_CONFIG,
        storage_config,
        ble_battery_config,
        ..Default::default()
    };

    // Keymap + storage
    let mut default_keymap = get_default_keymap();
    let mut encoder_keymap = get_default_encoder_map();
    let mut per_key_config = PositionalConfig::default();
    let (keymap, mut storage) = initialize_encoder_keymap_and_storage(
        &mut default_keymap,
        &mut encoder_keymap,
        flash,
        &rmk_config.storage_config,
        &mut behavior_config,
        &mut per_key_config,
    )
    .await;

    // Keyboard (processes events from peripherals)
    let mut keyboard = Keyboard::new(&keymap);

    // Empty matrix for dongle
    let direct_pins: [[Option<embassy_nrf::gpio::Input>; 0]; 0] = [];
    let debouncer = DefaultDebouncer::new();
    let matrix = rmk::direct_pin::DirectPinMatrix::<_, _, 0, 0, 0>::new(direct_pins, debouncer, true);
    let mut matrix = rmk::matrix::OffsetMatrixWrapper::<_, _, _, 0, 0>(matrix);

    // Battery ADC
    let saadc_config = saadc::Config::default();
    interrupt::SAADC.set_priority(interrupt::Priority::P3);
    let adc = Saadc::new(
        p.SAADC,
        Irqs,
        saadc_config,
        [saadc::ChannelConfig::single_ended(saadc::VddhDiv5Input.degrade_saadc())],
    );
    adc.calibrate().await;
    let mut adc_device = NrfAdc::new(
        adc,
        [AnalogEventType::Battery],
        embassy_time::Duration::from_secs(30),
        None,
    );
    let mut battery_processor = BatteryProcessor::new(1, 5, &keymap);

    // Peripheral addresses from storage
    let peripheral_addrs = rmk::split::ble::central::read_peripheral_addresses::<
        2, _, ROW, COL, NUM_LAYER, NUM_ENCODER,
    >(&mut storage)
    .await;

    // === JOYSTICK FIX: create JoystickProcessors on the central ===
    let mut joystick_l = JoystickProcessor::<ROW, COL, NUM_LAYER, NUM_ENCODER, 2>::new(
        [[80, 0], [0, 80]], [29130, 29365], 6, &keymap,
    );

    // Run everything
    join(
        rmk::join_all!(
            // Input devices + matrix
            {
                use rmk::input_device::InputDevice;
                rmk::join_all!(
                    async {
                        loop {
                            let e = adc_device.read_event().await;
                            match e {
                                rmk::event::Event::Key(key_event) => {
                                    rmk::channel::KEY_EVENT_CHANNEL.send(key_event).await;
                                }
                                _ => {
                                    if rmk::channel::EVENT_CHANNEL.is_full() {
                                        let _ = rmk::channel::EVENT_CHANNEL.receive().await;
                                    }
                                    rmk::channel::EVENT_CHANNEL.send(e).await;
                                }
                            }
                        }
                    },
                    async {
                        loop {
                            let e = matrix.read_event().await;
                            match e {
                                rmk::event::Event::Key(key_event) => {
                                    rmk::channel::KEY_EVENT_CHANNEL.send(key_event).await;
                                }
                                _ => {
                                    if rmk::channel::EVENT_CHANNEL.is_full() {
                                        let _ = rmk::channel::EVENT_CHANNEL.receive().await;
                                    }
                                    rmk::channel::EVENT_CHANNEL.send(e).await;
                                }
                            }
                        }
                    }
                )
            },
            keyboard.run(),
            rmk::run_rmk(&keymap, driver, &stack, &mut storage, rmk_config),
            {
                use rmk::input_device::InputProcessor;
                async {
                    loop {
                        let event = rmk::channel::EVENT_CHANNEL.receive().await;
                        let mut current_event = event;

                        // Battery processor
                        match battery_processor.process(current_event).await {
                            rmk::input_device::ProcessResult::Stop => continue,
                            rmk::input_device::ProcessResult::Continue(next_event) => {
                                current_event = next_event;
                            }
                        }

                        // Joystick processor (THE FIX)
                        match joystick_l.process(current_event).await {
                            rmk::input_device::ProcessResult::Stop => continue,
                            rmk::input_device::ProcessResult::Continue(_) => {}
                        }
                    }
                }
            }
        ),
        join(
            scan_peripherals(&stack, &peripheral_addrs),
            join(
                run_peripheral_manager::<5, 6, 0, 0, _>(0, &peripheral_addrs, &stack),
                run_peripheral_manager::<5, 6, 0, 6, _>(1, &peripheral_addrs, &stack),
            ),
        ),
    )
    .await;
}
