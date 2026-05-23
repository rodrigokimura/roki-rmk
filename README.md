# RoKi — RMK Firmware (Local Build)

Firmware for the RoKi split ergonomic keyboard, built locally with Rust + RMK.

## Build

```bash
cd rmk-local
RUST_MIN_STACK=67108864 cargo build --release
cargo make uf2 --release
```

Outputs:
- `RoKi-central.uf2` → dongle (USB/BLE central)
- `RoKi-peripheral.uf2` → left half
- `RoKi-peripheral2.uf2` → right half

## Flash

1. **Dongle** — double-tap reset → drag `RoKi-central.uf2` onto `NRF52BOOT`
2. **Left half** — double-tap reset → drag `RoKi-peripheral.uf2`
3. **Right half** — double-tap reset → drag `RoKi-peripheral2.uf2`

## Hardware

- **Controller**: nice!nano v2 (nRF52840)
- **Matrix**: 5×6 per half, direct pin
- **Encoders**: 1 per half (P0.17 / P0.20)
- **Joysticks**: 1 per half (P0.31 X / P0.29 Y), 45° per-side rotation
- **Buzzer**: piezo on P0.06 per half, R2-D2 connect/disconnect sounds

## Project Layout

```
├── keyboard.toml          # Hardware + keymap configuration
├── vial.json              # Vial layout descriptor
└── rmk-local/             # Local Rust firmware
    ├── src/
    │   ├── central.rs     # Dongle: BLE central, joystick processor
    │   ├── peripheral.rs  # Left half: matrix, encoder, joystick, buzzer
    │   ├── peripheral2.rs # Right half: same with CW joystick rotation
    │   └── keymap.rs      # Hardcoded keymap + VIAL config
    ├── keyboard.toml      # Synced copy from repo root
    ├── vial.json          # Synced copy from repo root
    └── Cargo.toml         # Dependencies + feature flags
```

## Key features

| Feature | Implementation |
|---------|---------------|
| BLE split | `rmk` built-in split peripheral/central |
| Joystick → mouse | Custom `#[controller(poll)]` on each half, per-side 45° rotation |
| Dead zone | Circular dead zone in peripheral joystick readers |
| Buzzer | `#[controller(event)]` on each half, R2-D2 tones on connect/disconnect |
| Battery | `BatteryProcessor` on central, VDDH ADC |

## Calibrating joysticks

Watch raw ADC values with a debug probe, then tune in `peripheral.rs` / `peripheral2.rs`:

```rust
const CENTER_X: i32 = 7500;  // resting ADC value
const CENTER_Y: i32 = 7500;
const SCALE: i32 = 64;       // sensitivity
const DEAD_ZONE: i32 = 4;    // raw mouse units
```

## Updating after `keyboard.toml` changes

1. Edit `keyboard.toml` at repo root
2. `cp ../keyboard.toml ../vial.json .` (in `rmk-local/`)
3. If keymap changed: manually update `src/keymap.rs` (hardcoded arrays)
4. Rebuild & reflash all three binaries

## References

- [RMK Docs](https://rmk.rs/docs/configuration.html)
- [RMK Split Keyboard](https://rmk.rs/docs/features/split_keyboard.html)
