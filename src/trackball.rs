//! RoKi trackball — split **peripheral 1**.
//!
//! 4-button matrix + PMW3360 optical sensor (RMK's `pmw33xx` driver).
//! Ships events to the left half (central) over BLE. Has no keymap and no
//! host link.
//!
//! Debugging the sensor wiring: build with `--no-default-features --features
//! usb_log` (`cargo make uf2-trackball-usblog`), plug the board into USB and
//! open the CDC serial port. The driver logs "PMW3360 detected, product ID:
//! 0x42" on success, or "Invalid product id" / "Init failed" otherwise.

#![no_std]
#![no_main]

mod fmt;
mod devices;

#[cfg(feature = "defmt")]
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Input, Level, Output, OutputDrive, Pull};
use embassy_nrf::mode::Async;
use embassy_nrf::peripherals::RNG;
#[cfg(feature = "usb_log")]
use embassy_nrf::peripherals::USBD;
#[cfg(feature = "usb_log")]
use embassy_nrf::usb;
use embassy_nrf::{bind_interrupts, rng, spim};
use nrf_mpsl::Flash;
use nrf_sdc::mpsl::MultiprotocolServiceLayer;
use nrf_sdc::{self as sdc, mpsl};
#[cfg(feature = "defmt")]
use panic_probe as _;

use fmt::{info, unwrap};
use rmk::config::StorageConfig;
use rmk::debounce::default_debouncer::DefaultDebouncer;
use rmk::futures::future::join;
use rmk::input_device::pmw33xx::{Pmw33xx, Pmw33xxConfig, Pmw3360Spec};
use rmk::input_device::pointing::PointingDevice;
use rmk::matrix::Matrix;
use rmk::run_all;
use rmk::split::peripheral::run_rmk_split_peripheral;
use rmk::storage::new_storage_without_keymap;
use rmk::watchdog::Nrf52Watchdog;
use static_cell::StaticCell;

use devices::PeripheralBuzzer;

#[cfg(not(feature = "usb_log"))]
bind_interrupts!(struct Irqs {
    TWISPI1 => spim::InterruptHandler<embassy_nrf::peripherals::TWISPI1>;
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

/// Debug builds: USB CDC serial port carrying the logs.
#[cfg(feature = "usb_log")]
#[embassy_executor::task]
async fn usb_log_task(
    driver: embassy_nrf::usb::Driver<'static, embassy_nrf::usb::vbus_detect::HardwareVbusDetect>,
) {
    rmk::usb::run_peripheral_usb(
        driver,
        rmk::config::DeviceConfig {
            vid: 0x4c4b,
            pid: 0x524e,
            manufacturer: "RoKi",
            product_name: "RoKi Trackball (debug)",
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
    info!("RoKi trackball (peripheral 1) starting");

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
        for i in (1..=15).rev() {
            embassy_time::Timer::after_secs(1).await;
            info!("RoKi trackball (debug): starting setup in {}s", i);
        }
        fmt::report_last_crash();
    }

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
    // 8704 is the value used by the right-half peripheral in this repo.
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

    // --- Storage (central's address bond) ---------------------------------
    let storage_config = StorageConfig {
        start_addr: 0xA0000,
        num_sectors: 32,
        ..Default::default()
    };
    let flash = Flash::take(mpsl, p.NVMC);
    let mut storage = new_storage_without_keymap(flash, storage_config).await;

    // --- Matrix (row2col) — pins per keyboard.toml [[split.peripheral]] ---
    let row_pins = [
        Output::new(p.P0_03, Level::Low, OutputDrive::Standard),
    ];
    let col_pins = [
        Input::new(p.P0_04, Pull::Down),
        Input::new(p.P0_28, Pull::Down),
        Input::new(p.P0_30, Pull::Down),
        Input::new(p.P0_05, Pull::Down),
    ];
    let debouncer = DefaultDebouncer::new();
    let mut matrix = Matrix::<_, _, _, 1, 4, false>::new(row_pins, col_pins, debouncer);

    // --- PMW3360 over SPI (mode 3, 2 MHz) --------------------------------
    //   SCK  P0.17   MOSI P0.20   MISO P0.29   NCS P1.13   MOTION P1.11
    let mut spim_config = spim::Config::default();
    spim_config.frequency = spim::Frequency::M2;
    spim_config.mode = spim::MODE_3;
    let spim = spim::Spim::new(p.TWISPI1, Irqs, p.P0_17, p.P0_29, p.P0_20, spim_config);
    let cs = Output::new(p.P1_13, Level::High, OutputDrive::Standard);
    // MOTION is open-drain active-low; leave `None` if not wired.
    let motion = Some(Input::new(p.P1_11, Pull::Up));

    // Device id 2: the joysticks on the halves are 0 (left) and 1 (right).
    // Deltas are forwarded as PointingEvents to the central's PointingProcessor.
    // 1600 CPI, tune in ±400 steps.
    // Report rate must stay <= ~125 Hz: the split BLE link runs a 7.5 ms
    // connection interval and sends one notification per PointingEvent, so a
    // faster rate overflows the event channel and the cursor stalls/jumps.
    let sensor_config = Pmw33xxConfig { res_cpi: 1600, ..Default::default() };
    let mut trackball =
        PointingDevice::<Pmw33xx<_, _, _, Pmw3360Spec>>::with_report_hz(2, spim, cs, motion, sensor_config, 125);

    // --- Buzzer (P0.06) — chirps when the link to the central comes/goes --
    let mut buzzer = PeripheralBuzzer::new(Output::new(p.P0_06, Level::Low, OutputDrive::Standard));

    info!("setup done, starting tasks");
    let mut watchdog_runner = Nrf52Watchdog::default_runner(p.WDT);

    join(
        run_all!(matrix, trackball, buzzer, storage, watchdog_runner),
        run_rmk_split_peripheral(1, sdc, ble_addr()),
    )
    .await;
}
