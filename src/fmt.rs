//! Logging shim: `defmt` (RTT) by default, `log` (USB CDC) with `--features usb_log`.
#![allow(unused_macros, unused_imports, dead_code)]

#[cfg(feature = "defmt")]
pub use defmt::{info, unwrap};

#[cfg(not(feature = "defmt"))]
pub use log::info;

#[cfg(not(feature = "defmt"))]
macro_rules! unwrap {
    ($e:expr) => {
        $e.unwrap()
    };
}
#[cfg(not(feature = "defmt"))]
pub(crate) use unwrap;

// === Crash recorder (usb_log builds only) ====================================
//
// The panic message is written to a RAM buffer that is *not* zeroed on reset
// (`.uninit`), then the chip resets. On the next boot `report_last_crash`
// prints the saved message and the nRF RESETREAS register, so a boot-loop
// can be diagnosed over the USB serial port.

#[cfg(not(feature = "defmt"))]
mod crash {
    use core::fmt::Write;

    const MAGIC: u32 = 0xC0DE_DEAD;
    const LEN: usize = 256;

    #[repr(C)]
    pub struct Record {
        magic: u32,
        len: u32,
        msg: [u8; LEN],
    }

    #[unsafe(link_section = ".uninit.CRASH")]
    static mut RECORD: Record = Record { magic: 0, len: 0, msg: [0; LEN] };

    struct Buf<'a> {
        buf: &'a mut [u8; LEN],
        pos: usize,
    }
    impl Write for Buf<'_> {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            for &b in s.as_bytes() {
                if self.pos < LEN {
                    self.buf[self.pos] = b;
                    self.pos += 1;
                }
            }
            Ok(())
        }
    }

    pub fn save(info: &core::panic::PanicInfo) {
        // SAFETY: called from the panic handler with interrupts about to die.
        let rec = unsafe { &mut *core::ptr::addr_of_mut!(RECORD) };
        let mut b = Buf { buf: &mut rec.msg, pos: 0 };
        let _ = write!(b, "{}", info);
        rec.len = b.pos as u32;
        rec.magic = MAGIC;
    }

    pub fn report() {
        let reas = embassy_nrf::pac::POWER.resetreas().read();
        log::warn!(
            "RESETREAS: resetpin={} dog={} sreq={} lockup={} off={} lpcomp={} dif={} vbus={}",
            reas.resetpin(),
            reas.dog(),
            reas.sreq(),
            reas.lockup(),
            reas.off(),
            reas.lpcomp(),
            reas.dif(),
            reas.vbus()
        );
        // Clear so the next boot shows only the newest reason.
        embassy_nrf::pac::POWER.resetreas().write_value(reas);

        // SAFETY: single-threaded at boot.
        let rec = unsafe { &mut *core::ptr::addr_of_mut!(RECORD) };
        if rec.magic == MAGIC {
            let len = (rec.len as usize).min(LEN);
            let msg = core::str::from_utf8(&rec.msg[..len]).unwrap_or("<non-utf8>");
            log::error!("LAST PANIC: {}", msg);
            rec.magic = 0;
        } else {
            log::info!("No panic recorded from previous boot");
        }
    }
}

/// Print the previous boot's reset reason and panic message (usb_log only).
#[cfg(not(feature = "defmt"))]
pub fn report_last_crash() {
    crash::report();
}
#[cfg(feature = "defmt")]
pub fn report_last_crash() {}

/// Without defmt there's no panic-probe: save the message and reset.
#[cfg(not(feature = "defmt"))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    crash::save(info);
    // Wait ~1.5 s before resetting: the Adafruit bootloader treats two resets
    // within ~500 ms as a "double tap" and would drop into DFU mode.
    cortex_m::asm::delay(96_000_000);
    cortex_m::peripheral::SCB::sys_reset();
}

/// HardFault (e.g. stack overflow caught by flip-link): record and reset.
#[cfg(not(feature = "defmt"))]
#[cortex_m_rt::exception]
unsafe fn HardFault(_ef: &cortex_m_rt::ExceptionFrame) -> ! {
    panic!("HardFault");
}
