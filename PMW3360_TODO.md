# TODO: Add PMW3360 Trackball Peripheral (Option 2)

## Goal
Add a 3rd nice!nano (`trackball.rs`) with a **PMW3360 optical sensor**, connected via SPI. It sends `Event::Joystick` (mouse deltas) to the central dongle over BLE split, processed by the existing `JoystickProcessor`.

---

## Step 0: Hardware Wiring

Connect the PMW3360 breakout to a **nice!nano v2**. Use a board that has the **SROM pre-loaded** (most commercial breakouts like Ploopy, JBO, etc. have an onboard MCU for this). A raw PMW3360 chip without SROM will NOT work.

| PMW3360 | nice!nano | Notes |
|---------|-----------|-------|
| VCC | 3V3 | Power |
| GND | GND | Ground |
| SCK | **P0.17** | SPI Clock (use any free GPIO) |
| MOSI | **P0.20** | SPI Data → Sensor |
| MISO | **P0.29** | SPI Data ← Sensor |
| CS | **P1.13** | Chip Select (active low) |

> **Important:** The PMW3360 requires **SPI Mode 3** (CPOL=1, CPHA=1). CS must have ≥120ns deassertion time between register accesses.

**Free pins on nice!nano not used by RoKi half** (verify your actual PCB first):
- `P0.03`, `P0.17`, `P0.20`, `P0.28`, `P0.29`, `P0.30`, `P1.01`, `P1.02`, `P1.13`, `P1.15`

Suggested wiring:
- SCK → `P0.17`
- MOSI → `P0.20`
- MISO → `P0.29`
- CS → `P1.13`

---

## Step 1: `keyboard.toml` — Add 3rd Peripheral

```toml
[rmk]
ble_profiles_num = 1
split_peripherals_num = 3

# ... existing left/right peripherals ...

[[split.peripheral]]
rows = 0
cols = 0
row_offset = 0
col_offset = 0
ble_addr = [0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc]
[split.peripheral.matrix]
matrix_type = "direct_pin"
direct_pins = []
```

> The trackball peripheral has no matrix rows/cols. The empty `direct_pins` works with the macro.

Also update the central peripheral count in `keyboard.toml` if there's a `split_peripherals_num` field.

---

## Step 2: `Cargo.toml` — Add Binary Target

```toml
[[bin]]
name = "trackball"
path = "src/trackball.rs"
```

---

## Step 3: Create `src/trackball.rs`

Use `#[rmk_peripheral(id = 2)]` with a custom `#[controller(poll)]` that:
- Initializes `embassy_nrf::spim::Spim` (async SPI, Mode 3)
- Reads motion deltas from PMW3360 registers
- Sends `Event::Joystick` to `EVENT_CHANNEL`

### Skeleton

```rust
#![no_main]
#![no_std]

use rmk::macros::rmk_peripheral;

// NOTE: Add SPI2 interrupt binding if using SPI2:
// embassy_nrf::bind_interrupts!(struct Irqs {
//     SPI2 => embassy_nrf::spim::InterruptHandler<embassy_nrf::peripherals::SPI2>;
// });
// Or use SPI1 / SPI0 depending on which pins you wire.

#[rmk_peripheral(id = 2)]
mod keyboard_peripheral {
    use rmk::controller::PollingController;

    #[controller(poll)]
    fn trackball_reader() {
        struct TrackballReader {
            spi: embassy_nrf::spim::Spim<'static, embassy_nrf::peripherals::SPI0>,
            cs: embassy_nrf::gpio::Output<'static>,
        }

        impl rmk::controller::Controller for TrackballReader {
            type Event = rmk::event::ControllerEvent;
            async fn process_event(&mut self, _event: Self::Event) {}
            async fn next_message(&mut self) -> Self::Event {
                core::future::pending().await
            }
        }

        impl PollingController for TrackballReader {
            const INTERVAL: embassy_time::Duration =
                embassy_time::Duration::from_hz(125); // 125 Hz report rate

            async fn update(&mut self) {
                // 1. Read motion register (0x02)
                //    Bit 7 = motion occurred
                // 2. If motion, read Delta X_L (0x03), X_H (0x04), Y_L (0x05), Y_H (0x06)
                // 3. Combine into i16 deltas
                // 4. Scale/sensitivity adjustment
                // 5. Send Event::Joystick with Rel deltas

                let dx = self.read_delta_x().await;
                let dy = self.read_delta_y().await;

                if dx == 0 && dy == 0 {
                    return;
                }

                let event = rmk::event::Event::Joystick([
                    rmk::event::AxisEvent {
                        typ: rmk::event::AxisValType::Rel,
                        axis: rmk::event::Axis::X,
                        value: dx,
                    },
                    rmk::event::AxisEvent {
                        typ: rmk::event::AxisValType::Rel,
                        axis: rmk::event::Axis::Y,
                        value: dy,
                    },
                    rmk::event::AxisEvent {
                        typ: rmk::event::AxisValType::Rel,
                        axis: rmk::event::Axis::Z,
                        value: 0,
                    },
                ]);

                if rmk::channel::EVENT_CHANNEL.is_full() {
                    let _ = rmk::channel::EVENT_CHANNEL.receive().await;
                }
                rmk::channel::EVENT_CHANNEL.send(event).await;
            }
        }

        impl TrackballReader {
            async fn read_reg(&mut self, reg: u8) -> u8 {
                // PMW3360 read protocol:
                // 1. CS low
                // 2. Send address byte (0x80 | reg)
                // 3. Send dummy byte (0x00) while receiving data
                // 4. CS high
                // 5. Delay ~120ns (or a few nops)
                // (Exact timing depends on your SPI implementation)
                todo!()
            }

            async fn write_reg(&mut self, reg: u8, val: u8) {
                // PMW3360 write protocol:
                // 1. CS low
                // 2. Send address byte (reg)
                // 3. Send data byte
                // 4. CS high
                // 5. Delay ~120ns
                todo!()
            }

            async fn read_delta_x(&mut self) -> i16 {
                todo!()
            }
            async fn read_delta_y(&mut self) -> i16 {
                todo!()
            }
        }

        // SPI config: Mode 3 (CPOL=1, CPHA=1), 2 MHz max
        let mut spim_config = embassy_nrf::spim::Config::default();
        spim_config.frequency = embassy_nrf::spim::Frequency::M2;
        spim_config.mode = embassy_nrf::spim::MODE_3;

        let spim = embassy_nrf::spim::Spim::new(
            p.SPI0, // or SPI1/SPI2 depending on pins
            Irqs,   // bind_interrupts! for the SPI peripheral
            p.P0_17, // SCK
            p.P0_29, // MISO
            p.P0_20, // MOSI
            spim_config,
        );

        let cs = embassy_nrf::gpio::Output::new(
            p.P1_13,
            embassy_nrf::gpio::Level::High, // PMW3360 CS active low
            embassy_nrf::gpio::OutputDrive::Standard,
        );

        let mut trackball = TrackballReader { spi: spim, cs };

        // --- PMW3360 init ---
        // 1. Read Product ID (reg 0x00) → should be 0x42 for PMW3360
        // 2. If 0x00 or wrong, delay 50ms and retry (SROM boot time)
        // 3. Set CPI via reg 0x0F (e.g., 0x0A = 1600 CPI)
        // 4. (Optional) Set lift-off distance via reg 0x63
        // 5. (Optional) Enable/Disable Rest mode via reg 0x40

        trackball
    }
}
```

---

## Step 4: `src/central.rs` — Add Peripheral Manager for id=2

Find this line:
```rust
let peripheral_addrs = rmk::split::ble::central::read_peripheral_addresses::<
    2, _, ROW, COL, NUM_LAYER, NUM_ENCODER,
>(&mut storage)
```

Change `2` → `3`:
```rust
let peripheral_addrs = rmk::split::ble::central::read_peripheral_addresses::<
    3, _, ROW, COL, NUM_LAYER, NUM_ENCODER,
>(&mut storage)
```

And add a third `run_peripheral_manager` in the join:
```rust
join(
    scan_peripherals(&stack, &peripheral_addrs),
    join(
        run_peripheral_manager::<5, 6, 0, 0, _>(0, &peripheral_addrs, &stack),
        join(
            run_peripheral_manager::<5, 6, 0, 6, _>(1, &peripheral_addrs, &stack),
            run_peripheral_manager::<0, 0, 0, 0, _>(2, &peripheral_addrs, &stack),
        ),
    ),
)
```

> Use `<0, 0, 0, 0, _>` for the trackball peripheral because it has no matrix.

---

## Step 5: `Makefile.toml` — Add Trackball Build Tasks

Add `objcopy-trackball` and `uf2-trackball` tasks, and update the `uf2` dependency list.

---

## Step 6: PMW3360 SPI Protocol Details

### Register Map (essential ones)

| Register | Address | Read/Write | Description |
|----------|---------|------------|-------------|
| Product_ID | 0x00 | R | Should be `0x42` for PMW3360 |
| Revision_ID | 0x01 | R | Should be `0x01` |
| Motion | 0x02 | R | Bit 7 = motion occurred since last read |
| Delta_X_L | 0x03 | R | X movement low byte (signed, 2's comp) |
| Delta_X_H | 0x04 | R | X movement high byte |
| Delta_Y_L | 0x05 | R | Y movement low byte |
| Delta_Y_H | 0x06 | R | Y movement high byte |
| SQUAL | 0x07 | R | Surface quality (0-169) |
| Config1 | 0x0F | R/W | CPI setting: `val * 100` CPI. Default `0x01` = 100 CPI. |
| Lift_Cutoff | 0x63 | R/W | Lift-off distance tuning |
| Motion_Burst | 0x50 | R | Burst read: motion + all deltas in one transaction |

### SPI Read Sequence (per register)

```
1. CS = LOW
2. MOSI: 0x80 | register_address  (MSB = 1 for read)
3. MOSI: 0x00 (dummy byte while MISO receives data)
4. CS = HIGH
5. DELAY ≥ 120ns (or ~1us for safety)
```

The dummy byte phase is when the sensor returns the register value on MISO.

### SPI Write Sequence

```
1. CS = LOW
2. MOSI: register_address (MSB = 0 for write)
3. MOSI: data_byte
4. CS = HIGH
5. DELAY ≥ 120ns
```

### Motion Burst (more efficient)

Instead of 5 separate reads (motion, x_l, x_h, y_l, y_h), send address `0x50` and read 12 bytes in one burst:
- Byte 0: Motion (0x02)
- Byte 1: Observation
- Byte 2: Delta_X_L
- Byte 3: Delta_X_H
- Byte 4: Delta_Y_L
- Byte 5: Delta_Y_H
- Byte 6: SQUAL
- Byte 7: RawData_Sum
- Byte 8: Max_RawData
- Byte 9: Min_RawData
- Byte 10: Shutter_Upper
- Byte 11: Shutter_Lower

> **Important:** After motion burst, you must wait ~35μs before reading another register, OR perform a dummy read of any register to reset the burst state machine.

### Initialization Checklist

1. **Power up:** The sensor needs ~50ms after power-on before responding to SPI (SROM upload time on breakout boards).
2. **Read Product_ID (0x00):** Expect `0x42`.
3. **Read Revision_ID (0x01):** Expect `0x01`.
4. **Set CPI:** Write `Config1` (0x0F). Most people use 800-1600 CPI.
   - `0x08` = 800 CPI
   - `0x0A` = 1000 CPI
   - `0x10` = 1600 CPI
5. **Optional tuning:** `Lift_Cutoff` (0x63) for `liftoff_dist`.

---

## Step 7: Build & Flash

```bash
RUST_MIN_STACK=67108864 cargo build --release --bin trackball
cargo make uf2 --release
# Flash RoKi-trackball.uf2 to the trackball nice!nano
```

The central (`RoKi-central.uf2`) also needs re-flashing since we changed `read_peripheral_addresses::<3, ...>`.

---

## Open Questions / Decisions to Make

1. **SPI instance:** Which `SPI0`/`SPI1`/`SPI2` to use? Check if any is already used by the macro-generated code. `keyboard.toml` uses no SPI devices, so any instance should be free.
2. **Pins:** Confirm the 4 pins (SCK, MOSI, MISO, CS) are free on your trackball nice!nano and not shorted to anything on your PCB.
3. **SROM:** Is your PMW3360 breakout pre-loaded with SROM? If not, you need to embed the SROM binary blob (about 4KB) and upload it over SPI at boot. This is non-trivial.
4. **CPI sensitivity:** What CPI value feels right for a thumb trackball? 800-1600 CPI is typical.
5. **Report rate:** 125 Hz (8ms) is standard for BLE HID mouse to avoid flooding. The central `JoystickProcessor` already handles this well.
6. **Motion pin:** Some PMW3360 breakouts expose a `MOTION` pin that goes low when motion is detected (allows interrupt-driven reads instead of polling). If available, you could use `embassy_nrf::gpio::Input::wait_for_low()` instead of polling at 125 Hz for better power efficiency. Otherwise, just poll via `PollingController` at 125 Hz.

---

## Reference Code: Minimal Embassy-nrf SPI Read

```rust
use embassy_nrf::{spim, gpio::{Output, Level, OutputDrive}};

async fn read_reg(spi: &mut spim::Spim<'_, embassy_nrf::peripherals::SPI0>, cs: &mut Output<'_>, reg: u8) -> u8 {
    cs.set_low();
    
    let tx = [0x80 | reg, 0x00]; // read address + dummy byte
    let mut rx = [0u8; 2];
    
    spi.transfer(&mut rx, &tx).await.unwrap();
    
    cs.set_high();
    
    embassy_time::Timer::after_micros(1).await; // 1us delay > 120ns requirement
    
    rx[1] // data returned during dummy byte
}
```

---

## Files to Touch Summary

| File | Changes |
|------|---------|
| `keyboard.toml` | Add `split_peripherals_num = 3`, add 3rd `[[split.peripheral]]` with `id=2` |
| `Cargo.toml` | Add `[[bin]] name = "trackball"` |
| `src/trackball.rs` | **New file** — SPI init + PMW3360 driver + motion loop |
| `src/central.rs` | Change `read_peripheral_addresses::<3`, add `run_peripheral_manager(2, ...)` |
| `Makefile.toml` | Add `objcopy-trackball` + `uf2-trackball` tasks |
| `README.md` | Document trackball wiring |

---

*Resume by implementing Step 3 (`src/trackball.rs`) first, then Steps 1, 2, 4, 5 in order.*
