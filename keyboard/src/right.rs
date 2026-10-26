//! RoKi right half — split **peripheral 0**.
//!
//! Scans its matrix, encoder and joystick and ships the events to the left
//! half (central) over BLE. Has no keymap and no host link.

#![no_std]
#![no_main]

mod fmt;
mod devices;

#[cfg(feature = "defmt")]
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Input, Level, Output, OutputDrive, Pull};
use embassy_nrf::interrupt::{self, InterruptExt};
use embassy_nrf::mode::Async;
use embassy_nrf::peripherals::RNG;
#[cfg(feature = "usb_log")]
use embassy_nrf::peripherals::USBD;
use embassy_nrf::saadc::{self, Input as _, Saadc};
#[cfg(feature = "usb_log")]
use embassy_nrf::usb;
use embassy_nrf::{bind_interrupts, rng};
use nrf_mpsl::Flash;
use nrf_sdc::mpsl::MultiprotocolServiceLayer;
use nrf_sdc::{self as sdc, mpsl};
#[cfg(feature = "defmt")]
use panic_probe as _;

use fmt::{info, unwrap};
use rmk::config::StorageConfig;
use rmk::debounce::default_debouncer::DefaultDebouncer;
use rmk::futures::future::join;
use rmk::input_device::battery::BatteryProcessor;
use rmk::input_device::rotary_encoder::RotaryEncoder;
use rmk::matrix::Matrix;
use rmk::matrix::direct_pin::DirectPinMatrix;
use rmk::run_all;
use rmk::split::peripheral::run_rmk_split_peripheral;
use rmk::storage::new_storage_without_keymap;
use rmk::watchdog::Nrf52Watchdog;
use static_cell::StaticCell;

use devices::{JoystickReader, PeripheralBuzzer, Rotation};

#[cfg(not(feature = "usb_log"))]
bind_interrupts!(struct Irqs {
    SAADC => saadc::InterruptHandler;
    RNG => rng::InterruptHandler<RNG>;
    EGU0_SWI0 => nrf_sdc::mpsl::LowPrioInterruptHandler;
    CLOCK_POWER => nrf_sdc::mpsl::ClockInterruptHandler;
    RADIO => nrf_sdc::mpsl::HighPrioInterruptHandler;
    TIMER0 => nrf_sdc::mpsl::HighPrioInterruptHandler;
    RTC0 => nrf_sdc::mpsl::HighPrioInterruptHandler;
});

#[cfg(feature = "usb_log")]
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

/// Debug builds: USB CDC serial port carrying the logs. Spawned first so it
/// enumerates even if later setup panics.
#[cfg(feature = "usb_log")]
#[embassy_executor::task]
async fn usb_log_task(
    driver: embassy_nrf::usb::Driver<'static, embassy_nrf::usb::vbus_detect::HardwareVbusDetect>,
) {
    rmk::usb::run_peripheral_usb(
        driver,
        rmk::config::DeviceConfig {
            vid: 0x4c4b,
            pid: 0x524d,
            manufacturer: "RoKi",
            product_name: "RoKi Right (debug)",
            ..rmk::config::DeviceConfig::default()
        },
    )
    .await
}

const L2CAP_TXQ: u8 = 3;
const L2CAP_RXQ: u8 = 3;
const L2CAP_MTU: usize = 251;

/// Peripheral-only controller: advertises to the central, never scans.
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
        .peripheral_count(1)?
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
    info!("RoKi right (peripheral 0) starting");

    // --- nRF init ---------------------------------------------------------
    let mut nrf_config = embassy_nrf::config::Config::default();
    nrf_config.dcdc.reg0_voltage = Some(embassy_nrf::config::Reg0Voltage::_3V3);
    nrf_config.dcdc.reg0 = false;
    nrf_config.dcdc.reg1 = false;
    let p = embassy_nrf::init(nrf_config);

    #[cfg(feature = "usb_log")]
    {
        let driver = embassy_nrf::usb::Driver::new(
            p.USBD,
            Irqs,
            embassy_nrf::usb::vbus_detect::HardwareVbusDetect::new(Irqs),
        );
        spawner.spawn(unwrap!(usb_log_task(driver)));
        // Give the serial monitor time to attach, then explain the last reset.
        for i in (1..=15).rev() {
            embassy_time::Timer::after_secs(1).await;
            info!("RoKi right (debug): starting setup in {}s", i);
        }
        fmt::report_last_crash();
    }

    // --- MPSL + SDC -------------------------------------------------------
    info!("[boot] init MPSL");
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
    info!("[boot] MPSL running");

    let sdc_p = sdc::Peripherals::new(
        p.PPI_CH17, p.PPI_CH18, p.PPI_CH20, p.PPI_CH21, p.PPI_CH22, p.PPI_CH23, p.PPI_CH24, p.PPI_CH25, p.PPI_CH26,
        p.PPI_CH27, p.PPI_CH28, p.PPI_CH29,
    );
    let mut rng = rng::Rng::new(p.RNG, Irqs);
    // NOTE: 4624 (the RMK example value) is too small for this controller
    // config (2M PHY + DLE) and makes `build_sdc` fail at boot.
    let mut sdc_mem = sdc::Mem::<8704>::new();
    let sdc = match build_sdc(sdc_p, &mut rng, mpsl, &mut sdc_mem) {
        Ok(sdc) => sdc,
        Err(e) => {
            #[cfg(not(feature = "defmt"))]
            log::error!("[boot] build_sdc failed: {:?}", e);
            #[cfg(not(feature = "defmt"))]
            embassy_time::Timer::after_secs(2).await;
            panic!("build_sdc failed: {:?}", e);
        }
    };
    info!("[boot] SDC built");

    // --- Storage (central's address bond) ---------------------------------
    let storage_config = StorageConfig {
        start_addr: 0xA0000,
        num_sectors: 32,
        ..Default::default()
    };
    let flash = Flash::take(mpsl, p.NVMC);
    let mut storage = new_storage_without_keymap(flash, storage_config).await;
    info!("[boot] storage ready");

    // --- Matrix (row2col) — pins per keyboard.toml [[split.peripheral]] ---
    let mut row_pins = [
        Output::new(p.P0_24, Level::Low, OutputDrive::Standard),
        Output::new(p.P1_00, Level::Low, OutputDrive::Standard),
        Output::new(p.P0_11, Level::Low, OutputDrive::Standard),
        Output::new(p.P1_04, Level::Low, OutputDrive::Standard),
        Output::new(p.P1_06, Level::Low, OutputDrive::Standard),
    ];
    let col_pins = [
        Input::new(p.P0_09, Pull::Down),
        Input::new(p.P0_10, Pull::Down),
        Input::new(p.P1_11, Pull::Down),
        Input::new(p.P1_13, Pull::Down),
        Input::new(p.P1_15, Pull::Down),
        Input::new(p.P0_02, Pull::Down),
    ];
    let direct_pin = Input::new(p.P0_22, Pull::Up);

    // --- Bootmagic (keyboard.toml `bootmagic = (0, 0)`, hand-implemented) ---
    // Same check RMK's macro generates: drive row 0, read col 0 — this
    // half's top-left key (the `6` key). The P0.22 direct key doubles as a
    // boot key. Hold either at power-on to drop into the bootloader.
    row_pins[0].set_high();
    embassy_time::Timer::after_micros(50).await;
    let boot_key_held = col_pins[0].is_high();
    row_pins[0].set_low();
    if boot_key_held || direct_pin.is_low() {
        rmk::boot::jump_to_bootloader();
    }

    let debouncer = DefaultDebouncer::new();
    // Local 5x6; the central places us at col_offset 6 (PeripheralMatrixConfig).
    let mut matrix = Matrix::<_, _, _, 5, 6, false>::new(row_pins, col_pins, debouncer);

    // --- Direct-pin key (P0.22 → local (5, 0), unified (5, 6)) -------------
    // Extra key wired straight to P0_22 (switch to GND), scanned alongside
    // the row/col matrix; the central adds col_offset 6 to the position.
    let mut direct_matrix = DirectPinMatrix::<_, _, 1, 1, 1, 5, 0>::new(
        [[Some(direct_pin)]],
        DefaultDebouncer::new(),
        true, // low active
    );

    // --- Encoder 1 --------------------------------------------------------
    let enc_a = Input::new(p.P0_17, Pull::Up);
    let enc_b = Input::new(p.P0_20, Pull::Up);
    let mut encoder = RotaryEncoder::with_resolution(enc_a, enc_b, 4, false, 1);

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
    info!("[boot] ADC calibrated");
    let mut joystick = JoystickReader::new(adc, Rotation::Cw45, 1);
    // Converts our ADC reading to a BatteryStatusEvent, which the split
    // peripheral forwards to the central.
    let mut battery_processor = BatteryProcessor::new(1, 5);

    // --- Buzzer (P0.06) ---------------------------------------------------
    let mut buzzer = PeripheralBuzzer::new(Output::new(p.P0_06, Level::Low, OutputDrive::Standard));

    info!("setup done, starting tasks");
    let mut watchdog_runner = Nrf52Watchdog::default_runner(p.WDT);

    join(
        run_all!(matrix, direct_matrix, encoder, joystick, battery_processor, buzzer, storage, watchdog_runner),
        run_rmk_split_peripheral(0, sdc, ble_addr()),
    )
    .await;
}
