//! RoKi Trackball — standalone BLE/USB mouse on a nice!nano v2.
//!
//! PMW3360 optical sensor (RMK's `pmw33xx` driver) + 4 buttons. Not part of
//! the keyboard's split: it bonds to hosts directly on one of 3 BLE profiles,
//! or works over USB when plugged in. Hold the outer button for layer 1 and
//! tap a profile key to switch (hold 5 s to clear that bond and re-pair).
//!
//! Debugging the sensor wiring: build with `--no-default-features --features
//! usb_log` (`cargo make uf2-trackball-usblog`), plug the board into USB and
//! open the CDC serial port. The driver logs "PMW3360 detected, product ID:
//! 0x42" on success, or "Invalid product id" / "Init failed" otherwise.

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
use embassy_nrf::{bind_interrupts, rng, spim, usb};
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
use rmk::input_device::adc::{AnalogEventType, NrfAdc};
use rmk::input_device::battery::BatteryProcessor;
use rmk::input_device::pmw33xx::{Pmw33xx, Pmw33xxConfig, Pmw3360Spec};
use rmk::input_device::pointing::{PointingDevice, PointingProcessor, PointingProcessorConfig};
use rmk::keyboard::Keyboard;
use rmk::matrix::Matrix;
use rmk::usb::UsbTransport;
use rmk::watchdog::Nrf52Watchdog;
use rmk::{KeymapData, initialize_keymap_and_storage, run_all};
use static_cell::StaticCell;

use devices::HostBuzzer;
use keymap::{KEYBOARD_DEVICE_CONFIG, VIAL_CONFIG};

bind_interrupts!(struct Irqs {
    USBD => usb::InterruptHandler<USBD>;
    SAADC => saadc::InterruptHandler;
    TWISPI1 => spim::InterruptHandler<embassy_nrf::peripherals::TWISPI1>;
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

/// Peripheral-only controller: advertises to hosts, never scans.
fn build_sdc<'d, const N: usize>(
    p: nrf_sdc::Peripherals<'d>,
    rng: &'d mut rng::Rng<Async>,
    mpsl: &'d MultiprotocolServiceLayer,
    mem: &'d mut sdc::Mem<N>,
) -> Result<nrf_sdc::SoftdeviceController<'d>, nrf_sdc::Error> {
    sdc::Builder::new()?
        .support_adv()
        .support_peripheral()
        .support_dle_peripheral()
        .support_phy_update_peripheral()
        .support_le_2m_phy()
        .peripheral_count(1)? // one host link
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
    info!("RoKi trackball starting");

    // --- nRF init ---------------------------------------------------------
    let mut nrf_config = embassy_nrf::config::Config::default();
    nrf_config.dcdc.reg0_voltage = Some(embassy_nrf::config::Reg0Voltage::_3V3);
    nrf_config.dcdc.reg0 = false;
    nrf_config.dcdc.reg1 = false;
    let p = embassy_nrf::init(nrf_config);
    fmt::report_last_crash();

    // --- USB --------------------------------------------------------------
    // Also carries the logs in `usb_log` builds (via rmk's UsbTransport).
    let driver = Driver::new(p.USBD, Irqs, HardwareVbusDetect::new(Irqs));

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
    let mut sdc_mem = sdc::Mem::<8704>::new();
    let sdc = unwrap!(build_sdc(sdc_p, &mut rng, mpsl, &mut sdc_mem));

    // --- Flash / storage --------------------------------------------------
    let storage_config = StorageConfig {
        start_addr: 0xA0000,
        num_sectors: 32,
        clear_storage: false,
        clear_layout: false,
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
    let mut keymap_data = KeymapData::new(keymap::get_default_keymap());
    let mut behavior_config = BehaviorConfig::default();
    let per_key_config = PositionalConfig::default();
    let (keymap, mut storage) =
        initialize_keymap_and_storage(&mut keymap_data, flash, &storage_config, &mut behavior_config, &per_key_config)
            .await;

    // --- Buttons: 1x4 row2col matrix — pins per keyboard.toml [matrix] -----
    let row_pins = [Output::new(p.P0_03, Level::Low, OutputDrive::Standard)];
    let col_pins = [
        Input::new(p.P0_04, Pull::Down),
        Input::new(p.P0_28, Pull::Down),
        Input::new(p.P0_30, Pull::Down),
        Input::new(p.P0_05, Pull::Down),
    ];
    let debouncer = DefaultDebouncer::new();
    let mut matrix = Matrix::<_, _, _, 1, 4, false, 0, 0>::new(row_pins, col_pins, debouncer);

    // --- PMW3360 over SPI (mode 3, 2 MHz) --------------------------------
    //   SCK P0.17   MOSI P0.20   MISO P0.29   NCS P1.13   MOTION P1.11
    let mut spim_config = spim::Config::default();
    spim_config.frequency = spim::Frequency::M2;
    spim_config.mode = spim::MODE_3;
    let spim = spim::Spim::new(p.TWISPI1, Irqs, p.P0_17, p.P0_29, p.P0_20, spim_config);
    let cs = Output::new(p.P1_13, Level::High, OutputDrive::Standard);
    // MOTION (P1.11) is deliberately NOT used: with it the driver sleeps on
    // `wait_for_low` and cursor movement was laggy — the module's MOTION pad
    // does not behave like the raw open-drain sensor pin (likely buffered /
    // pulled mid-rail). Timer polling every 500 µs is smooth and the extra
    // idle power is negligible next to the sensor LED. Only re-enable after
    // verifying the pad idles high (> 2.3 V) and pulls to ~0 V while rolling.
    let motion: Option<Input<'static>> = None;
    let _ = Input::new(p.P1_11, Pull::Up); // keep the pad pulled up while unused

    // 1600 CPI, tune in ±400 steps. 125 Hz report rate (one HID report per
    // BLE connection interval).
    let sensor_config = Pmw33xxConfig { res_cpi: 1600, ..Default::default() };
    let mut trackball =
        PointingDevice::<Pmw33xx<_, _, _, Pmw3360Spec>>::with_report_hz(0, spim, cs, motion, sensor_config, 125);
    let mut pointing_processor = PointingProcessor::new(&keymap, PointingProcessorConfig::default());

    // --- Battery (VDDH/5) --------------------------------------------------
    interrupt::SAADC.set_priority(interrupt::Priority::P3);
    let adc = Saadc::new(
        p.SAADC,
        Irqs,
        saadc::Config::default(),
        [saadc::ChannelConfig::single_ended(saadc::VddhDiv5Input.degrade_saadc())],
    );
    adc.calibrate().await;
    let mut battery_adc = NrfAdc::new(
        adc,
        [AnalogEventType::Battery],
        [0],
        embassy_time::Duration::from_secs(30),
        None,
    );
    // VDDH/5 is auto-detected by BatteryProcessor from the value range.
    let mut battery_processor = BatteryProcessor::new(1, 5);

    // --- Buzzer (P0.06) ---------------------------------------------------
    let mut buzzer = HostBuzzer::new(Output::new(p.P0_06, Level::Low, OutputDrive::Standard));

    // --- Keyboard core + transports ---------------------------------------
    let mut keyboard = Keyboard::new(&keymap);
    let host_service = HostService::new(&keymap, &rmk_config);
    let mut usb_transport = UsbTransport::new(driver, rmk_config.device_config).with_host_service(&host_service);
    let mut ble_transport = BleTransport::new(sdc, ble_addr(), rmk_config).with_host_service(&host_service);

    let mut watchdog_runner = Nrf52Watchdog::default_runner(p.WDT);

    info!("setup done, starting tasks");
    run_all!(
        matrix,
        trackball,
        pointing_processor,
        battery_adc,
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
