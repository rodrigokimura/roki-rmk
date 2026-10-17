# TODO: PMW3360 trackball as a third split peripheral

Add a 4th nice!nano carrying a **PMW3360** optical sensor. It joins the split
as **peripheral 1** and sends mouse deltas to the left half (central), which
already runs a `PointingProcessor`, so the dongle/host side needs no change.

```
 trackball (peripheral 1) ──BLE split──▶ left half (central) ──▶ dongle / BLE host
 right half (peripheral 0) ─BLE split──▶
```

RMK v0.9 ships a PMW3360/PMW3389 driver (`rmk::input_device::pmw33xx`) with
SROM upload, burst reads and a motion-pin fast path, so no hand-written SPI
code is needed.

## Hardware

| PMW3360 | nice!nano | Notes |
|---------|-----------|-------|
| VCC  | 3V3   | |
| GND  | GND   | |
| SCK  | P0.17 | any free GPIO |
| MOSI | P0.20 | |
| MISO | P0.29 | |
| CS   | P1.13 | active low |
| MOTION | P1.11 (optional) | interrupt-driven reads instead of polling |

SPI mode 3, ≤ 2 MHz. If the breakout has no onboard MCU/SROM, use
`Pmw33xx::new_with_firmware(..)` with the 4 KB SROM blob (`srom_firmware`);
otherwise `Pmw33xx::new(..)`.

## Changes

### `keyboard.toml`

```toml
[rmk]
split_peripherals_num = 2        # right half + trackball

# Trackball: no keys, so a 0×0 region placed anywhere outside the right half.
[[split.peripheral]]
rows = 0
cols = 0
row_offset = 0
col_offset = 0
[split.peripheral.matrix]
matrix_type = "direct_pin"
direct_pins = []

[event.pointing]
subs = 4                         # one more subscriber for the extra split link
```

The validator requires each board's pin count to match `rows`/`cols`, which an
empty `direct_pins` satisfies. Check it doesn't overlap `[split.central]`
(rows 0–4, cols 0–5) — a 0×0 region never does.

### `Cargo.toml` / `Makefile.toml`

Add `[[bin]] name = "trackball" path = "src/trackball.rs"` and
`objcopy-trackball` / `uf2-trackball` tasks mirroring the `right` ones; add
`uf2-trackball` to the `uf2` task.

### `src/trackball.rs`

Copy `src/right.rs` and replace matrix/encoder/joystick with:

```rust
use embassy_nrf::spim::{self, Spim};
use rmk::input_device::pmw33xx::{Pmw33xx, Pmw33xxConfig, Pmw3360Spec};
use rmk::input_device::pointing::PointingDevice;

bind_interrupts!(struct Irqs {
    SPI2 => spim::InterruptHandler<embassy_nrf::peripherals::SPI2>;
    // ... plus the SAADC/RNG/MPSL handlers from right.rs
});

let mut spi_config = spim::Config::default();
spi_config.frequency = spim::Frequency::M2;
spi_config.mode = spim::MODE_3;
let spi = Spim::new(p.SPI2, Irqs, p.P0_17, p.P0_29, p.P0_20, spi_config);
let cs = Output::new(p.P1_13, Level::High, OutputDrive::Standard);
let motion = Some(Input::new(p.P1_11, Pull::Up));   // or None

let config = Pmw33xxConfig { res_cpi: 1200, ..Default::default() };
// device id 2 — the joysticks are 0 (left) and 1 (right)
let mut trackball =
    PointingDevice::<Pmw33xx<_, _, _, Pmw3360Spec>>::with_report_hz(2, spi, cs, motion, config, 125);

// no matrix: the split peripheral still needs its storage + the BLE task
join(
    run_all!(trackball, battery_processor, storage, watchdog_runner),
    run_rmk_split_peripheral(1, sdc, ble_addr()),
)
.await;
```

Keep the `sdc::Mem::<8704>` size from `right.rs` — the 4624 from the RMK
example is too small for this controller config and fails silently.

### `src/left.rs`

`BleTransport::new` takes one `PeripheralMatrixConfig` per peripheral:

```rust
[
    PeripheralMatrixConfig { rows: 5, cols: 6, row_offset: 0, col_offset: 6 }, // right half
    PeripheralMatrixConfig { rows: 0, cols: 0, row_offset: 0, col_offset: 0 }, // trackball
]
```

Optionally give the trackball its own `PointingProcessor` (e.g. different
`invert_y`, or scroll mode via `set_pointing_mode`) by filtering on
`PointingProcessorConfig { device_id: 2, .. }` and running it alongside the
existing pass-through one.

## Open questions

1. SROM: pre-loaded breakout (Ploopy/JBO style) or raw sensor needing the blob?
2. CPI: 800–1600 is typical for a thumb trackball; start at 1200.
3. Motion pin wired or not? Without it the driver polls; with it the device
   sleeps between movements, which matters on battery.
4. Battery: does the trackball board have a battery, or is it USB-powered?
   If USB-only, drop `battery_processor`/ADC and the VDDH channel.
