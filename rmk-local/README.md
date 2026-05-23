# RoKi Local Firmware

Hand-written RMK firmware for the RoKi split keyboard. All logic is in the Rust source files — no cloud compilation, no macro-generated boilerplate.

## File layout

| File | Role |
|------|------|
| `src/central.rs` | Dongle: BLE central, battery processor, pass-through joystick processor |
| `src/keymap.rs` | Hardcoded keymap, encoder map, VIAL config |
| `src/peripheral.rs` | Left half: matrix, encoder, joystick ADC reader (CCW 45°), buzzer |
| `src/peripheral2.rs` | Right half: same with joystick CW 45° rotation |
| `keyboard.toml` | Copied from repo root (pin config for macro expansion) |
| `vial.json` | Copied from repo root |

## Build

```bash
cd rmk-local
RUST_MIN_STACK=67108864 cargo build --release
cargo make uf2 --release
```

## Updating after keyboard.toml changes

```bash
cp ../keyboard.toml ../vial.json .
RUST_MIN_STACK=67108864 cargo build --release
cargo make uf2 --release
```

If keymap or encoder map changed, also edit `src/keymap.rs` manually.
