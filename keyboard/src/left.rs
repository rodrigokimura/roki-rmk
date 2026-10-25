//! RoKi left half — split **central** and the keyboard itself.
//!
//! Talks BLE to the right half (peripheral 0) and to the host: either a BLE
//! profile directly, or the RoKi dongle via the v0.9 `dongle` bond slot.
//! Press `SwitchToDongle` (layer 1, right thumb cluster, outer key) to pair with the dongle.

#![no_std]
#![no_main]

mod devices;
mod fmt;
mod keymap;

#[cfg(feature = "defmt")]
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Input, Level, Output, OutputDrive, Pull};
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
#[cfg(feature = "defmt")]
use panic_probe as _;

use fmt::{info, unwrap};
use rmk::ble::BleTransport;
use rmk::config::{BehaviorConfig, BleBatteryConfig, PositionalConfig, RmkConfig, StorageConfig};
use rmk::debounce::default_debouncer::DefaultDebouncer;
use rmk::host::HostService;
use rmk::input_device::battery::BatteryProcessor;
use rmk::input_device::pointing::{PointingProcessor, PointingProcessorConfig};
use rmk::input_device::rotary_encoder::RotaryEncoder;
use rmk::keyboard::Keyboard;
use rmk::matrix::Matrix;
use rmk::matrix::direct_pin::DirectPinMatrix;
use rmk::split::PeripheralMatrixConfig;
use rmk::usb::UsbTransport;
use rmk::watchdog::Nrf52Watchdog;
use rmk::{KeymapData, initialize_keymap_and_storage, run_all};
use static_cell::StaticCell;

use devices::{CentralBuzzer, JoystickReader, Rotation};
use keymap::{KEYBOARD_DEVICE_CONFIG, VIAL_CONFIG};

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

const L2CAP_TXQ: u8 = 3;
const L2CAP_RXQ: u8 = 3;
const L2CAP_MTU: usize = 251;

/// Central toward the right half, peripheral toward the host/dongle.
fn build_sdc<'d, const N: usize>(
    p: nrf_sdc::Peripherals<'d>,
    rng: &'d mut rng::Rng<Async>,
    mpsl: &'d MultiprotocolServiceLayer,
    mem: &'d mut sdc::Mem<N>,
) -> Result<nrf_sdc::SoftdeviceController<'d>, nrf_sdc::Error> {
    sdc::Builder::new()?
        .support_scan()
        .support_central()
        .support_adv()
        .support_peripheral()
        .support_dle_peripheral()
        .support_dle_central()
        .support_phy_update_central()
        .support_phy_update_peripheral()
        .support_le_2m_phy()
        .central_count(1)? // one split peripheral (right half)
        .peripheral_count(1)? // one host link (BLE profile or dongle)
        .buffer_cfg(L2CAP_MTU as u16, L2CAP_MTU as u16, L2CAP_TXQ, L2CAP_RXQ)?
        .build(p, rng, mpsl, mem)
}

fn ble_addr() -> [u8; 6] {
    let ficr = embassy_nrf::pac::FICR;
    let high = u64::from(ficr.deviceid(1).read());
    let addr = high << 32 | u64::from(ficr.deviceid(0).read());
    let addr = addr | 0x0000_c000_0000_0000;
    unwrap!(addr.to_le_bytes()[..6].try_into())
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("RoKi left (central) starting");

    // --- nRF init ---------------------------------------------------------
    let mut nrf_config = embassy_nrf::config::Config::default();
    nrf_config.dcdc.reg0_voltage = Some(embassy_nrf::config::Reg0Voltage::_3V3);
    nrf_config.dcdc.reg0 = false;
    nrf_config.dcdc.reg1 = false;
    let p = embassy_nrf::init(nrf_config);

    // --- MPSL + SDC -------------------------------------------------------
    let mpsl_p = mpsl::Peripherals::new(p.RTC0, p.TIMER0, p.TEMP, p.PPI_CH19, p.PPI_CH30, p.PPI_CH31);
    let lfclk_cfg = mpsl::raw::mpsl_clock_lfclk_cfg_t {
        source: mpsl::raw::MPSL_CLOCK_LF_SRC_RC as u8,
        rc_ctiv: mpsl::raw::MPSL_RECOMMENDED_RC_CTIV as u8,
        rc_temp_ctiv: mpsl::raw::MPSL_RECOMMENDED_RC_TEMP_CTIV as u8,
        accuracy_ppm: 500,
        skip_wait_lfclk_started: mpsl::raw::MPSL_DEFAULT_SKIP_WAIT_LFCLK_STARTED != 0,
    };
    static MPSL: StaticCell<MultiprotocolServiceLayer> = StaticCell::new();
    static SESSION_MEM: StaticCell<mpsl::SessionMem<1>> = StaticCell::new();
    let mpsl = MPSL.init(unwrap!(mpsl::MultiprotocolServiceLayer::with_timeslots(
        mpsl_p,
        Irqs,
        lfclk_cfg,
        SESSION_MEM.init(mpsl::SessionMem::new())
    )));
    spawner.spawn(unwrap!(mpsl_task(&*mpsl)));

    let sdc_p = sdc::Peripherals::new(
        p.PPI_CH17, p.PPI_CH18, p.PPI_CH20, p.PPI_CH21, p.PPI_CH22, p.PPI_CH23, p.PPI_CH24, p.PPI_CH25, p.PPI_CH26,
        p.PPI_CH27, p.PPI_CH28, p.PPI_CH29,
    );
    let mut rng = rng::Rng::new(p.RNG, Irqs);
    let mut sdc_mem = sdc::Mem::<15472>::new();
    let sdc = unwrap!(build_sdc(sdc_p, &mut rng, mpsl, &mut sdc_mem));

    // --- USB --------------------------------------------------------------
    let driver = Driver::new(p.USBD, Irqs, HardwareVbusDetect::new(Irqs));

    // --- Flash / storage --------------------------------------------------
    let storage_config = StorageConfig {
        start_addr: 0xA0000,
        num_sectors: 32,
        clear_storage: false,
        clear_layout: true,
    };
    let flash = Flash::take(mpsl, p.NVMC);

    // --- RMK config -------------------------------------------------------
    let rmk_config = RmkConfig {
        device_config: KEYBOARD_DEVICE_CONFIG,
        vial_config: VIAL_CONFIG,
        storage_config,
        ble_battery_config: BleBatteryConfig::new(None, false, None, false),
    };

    // --- Keymap + storage -------------------------------------------------
    let mut keymap_data =
        KeymapData::new_with_encoder(keymap::get_default_keymap(), keymap::get_default_encoder_map());
    let mut behavior_config = BehaviorConfig::default();
    let per_key_config = PositionalConfig::default();
    let (keymap, mut storage) =
        initialize_keymap_and_storage(&mut keymap_data, flash, &storage_config, &mut behavior_config, &per_key_config)
            .await;

    // --- Matrix (row2col: rows are outputs, cols are inputs) --------------
    // Pins per keyboard.toml [split.central.matrix]
    let row_pins = [
        Output::new(p.P0_24, Level::Low, OutputDrive::Standard),
        Output::new(p.P1_00, Level::Low, OutputDrive::Standard),
        Output::new(p.P0_11, Level::Low, OutputDrive::Standard),
        Output::new(p.P1_04, Level::Low, OutputDrive::Standard),
        Output::new(p.P1_06, Level::Low, OutputDrive::Standard),
    ];
    let col_pins = [
        Input::new(p.P0_02, Pull::Down),
        Input::new(p.P1_15, Pull::Down),
        Input::new(p.P1_13, Pull::Down),
        Input::new(p.P1_11, Pull::Down),
        Input::new(p.P0_10, Pull::Down),
        Input::new(p.P0_09, Pull::Down),
    ];
    let debouncer = DefaultDebouncer::new();
    // ROW=5, COL=6, COL2ROW=false (row2col), offsets (0, 0)
    let mut matrix = Matrix::<_, _, _, 5, 6, false, 0, 0>::new(row_pins, col_pins, debouncer);

    // --- Direct-pin key (P0.22 → unified position (5, 0)) ------------------
    // Extra key wired straight to P0_22 (switch to GND), scanned alongside
    // the row/col matrix; see keyboard.toml [split.central.matrix].
    let mut direct_matrix = DirectPinMatrix::<_, _, 1, 1, 1, 5, 0>::new(
        [[Some(Input::new(p.P0_22, Pull::Up))]],
        DefaultDebouncer::new(),
        true, // low active
    );

    // --- Encoder 0 --------------------------------------------------------
    let enc_a = Input::new(p.P0_17, Pull::Up);
    let enc_b = Input::new(p.P0_20, Pull::Up);
    let mut encoder = RotaryEncoder::with_resolution(enc_a, enc_b, 4, false, 0);

    // --- ADC: joystick (P0.31 X, P0.29 Y) + battery (VDDH/5) --------------
    interrupt::SAADC.set_priority(interrupt::Priority::P3);
    let adc = Saadc::new(
        p.SAADC,
        Irqs,
        saadc::Config::default(),
        [
            saadc::ChannelConfig::single_ended(p.P0_31.degrade_saadc()),
            saadc::ChannelConfig::single_ended(p.P0_29.degrade_saadc()),
            saadc::ChannelConfig::single_ended(saadc::VddhDiv5Input.degrade_saadc()),
        ],
    );
    adc.calibrate().await;
    let mut joystick = JoystickReader::new(adc, Rotation::Ccw45, 0);
    // VDDH/5 is auto-detected by BatteryProcessor from the value range.
    let mut battery_processor = BatteryProcessor::new(1, 5);

    // Pass-through mouse: peripherals (and this half) already emit scaled deltas.
    let mut pointing_processor = PointingProcessor::new(&keymap, PointingProcessorConfig::default());

    // --- Buzzer (P0.06) ---------------------------------------------------
    let mut buzzer = CentralBuzzer::new(Output::new(p.P0_06, Level::Low, OutputDrive::Standard));

    // --- Keyboard core + transports ---------------------------------------
    let mut keyboard = Keyboard::new(&keymap);
    let host_service = HostService::new(&keymap, &rmk_config);
    let mut usb_transport = UsbTransport::new(driver, rmk_config.device_config).with_host_service(&host_service);
    // Right half: 5x6 at column offset 6
    let mut ble_transport = BleTransport::new(
        sdc,
        ble_addr(),
        rmk_config,
        [PeripheralMatrixConfig { rows: 6, cols: 6, row_offset: 0, col_offset: 6 }],
    )
    .with_host_service(&host_service);

    let mut watchdog_runner = Nrf52Watchdog::default_runner(p.WDT);

    run_all!(
        matrix,
        direct_matrix,
        encoder,
        joystick,
        pointing_processor,
        battery_processor,
        buzzer,
        storage,
        usb_transport,
        ble_transport,
        keyboard,
        watchdog_runner
    )
    .await;
}
