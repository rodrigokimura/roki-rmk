# RoKi — RMK Firmware

Hand-written firmware for the RoKi split ergonomic keyboard.

## Hardware

- **Controller**: nice!nano v2 (nRF52840)
- **Matrix**: 5×6 per half, direct pin
- **Encoders**: 1 per half (P0.17 / P0.20)
- **Joysticks**: 1 per half (P0.31 X / P0.29 Y), 45° per-side rotation
- **Buzzer**: piezo on P0.06 per half, R2-D2 connect/disconnect sounds

## File layout

| File | Role |
|------|------|
| `src/central.rs` | Dongle: BLE central, battery processor, pass-through joystick processor |
| `src/keymap.rs` | Hardcoded keymap, encoder map, VIAL config |
| `src/left.rs` | Left half: matrix, encoder, joystick ADC reader (CCW 45°), buzzer |
| `src/right.rs` | Right half: same with joystick CW 45° rotation |
| `keyboard.toml` | Hardware config (pins, matrix, split layout) |
| `vial.json` | Vial GUI layout descriptor |

## Build

```bash
RUST_MIN_STACK=67108864 cargo build --release
cargo make uf2 --release
```

Outputs:
- `RoKi-central.uf2` → dongle
- `RoKi-left.uf2` → left half
- `RoKi-right.uf2` → right half

## Flash

1. **Dongle** — double-tap reset → drag `RoKi-central.uf2` onto `NRF52BOOT`
2. **Left half** — double-tap reset → drag `RoKi-left.uf2`
3. **Right half** — double-tap reset → drag `RoKi-right.uf2`

## Calibrating joysticks

Watch raw ADC values with a debug probe, then tune in `src/left.rs` / `src/right.rs`:

```rust
const CENTER_X: i32 = 7500;  // resting ADC value
const CENTER_Y: i32 = 7500;
const SCALE: i32 = 64;       // sensitivity
const DEAD_ZONE: i32 = 4;    // raw mouse units
```

## Updating after keyboard.toml changes

Edit `keyboard.toml` directly at the repo root, then rebuild.

If keymap or encoder map changed, also edit `src/keymap.rs` manually (hardcoded Rust arrays extracted from macro expansion).

## Key features

| Feature | Implementation |
|---------|---------------|
| BLE split | `rmk` built-in split peripheral/central |
| Joystick → mouse | Custom `#[controller(poll)]` on each half, per-side 45° rotation |
| Dead zone | Circular dead zone in per-half joystick readers |
| Buzzer | `#[controller(event)]` on each half, R2-D2 tones on connect/disconnect |
| Battery | `BatteryProcessor` on central, VDDH ADC |

## References

- [RMK Docs](https://rmk.rs/docs/configuration.html)
- [RMK Split Keyboard](https://rmk.rs/docs/features/split_keyboard.html)
