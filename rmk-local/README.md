# RoKi Local Compilation (Joystick Fix)

This folder contains a hand-patched RMK project that fixes joystick support
for BLE split peripherals. The cloud-compiled `rmkit create` output places
`JoystickProcessor` on the peripheral (which has no HID transport), so mouse
reports are dropped. This version moves the processor to the central (dongle).

## File layout

| File | Source | Notes |
|------|--------|-------|
| `src/central.rs` | Hand-written | Explicit task setup; chains `BatteryProcessor` → `JoystickProcessor` on `EVENT_CHANNEL` |
| `src/keymap.rs` | Extracted from macro expansion | Hardcoded keymap, encoder map, VIAL config |
| `src/peripheral.rs` | `rmkit create` + macro | `#[rmk_peripheral(id = 0)]` reads `keyboard.toml` at compile time |
| `src/peripheral2.rs` | `rmkit create` + macro | `#[rmk_peripheral(id = 1)]` reads `keyboard.toml` at compile time |
| `keyboard.toml` | Copied from repo root | Must be kept in sync with root `keyboard.toml` |
| `vial.json` | Copied from repo root | Must be kept in sync with root `vial.json` |

## Build

```bash
cd rmk-local
RUST_MIN_STACK=67108864 cargo build --release
cargo make uf2 --release
```

Outputs:
- `RoKi-central.uf2` → dongle
- `RoKi-peripheral.uf2` → left half
- `RoKi-peripheral2.uf2` → right half

## Updating after `keyboard.toml` changes

### Case 1: Hardware / pins / BLE addresses / matrix (no keymap changes)

```bash
cp ../keyboard.toml ../vial.json .
RUST_MIN_STACK=67108864 cargo build --release
cargo make uf2 --release
```

The peripheral macros (`#[rmk_peripheral]`) read `keyboard.toml` at compile time,
so pin changes, BLE addresses, matrix config, etc. are picked up automatically.

### Case 2: Keymap or encoder map changes

`src/keymap.rs` contains hardcoded Rust arrays extracted from the macro-expanded
code. You must update it manually to match the new `keyboard.toml` layout.

**Option A — Manual edit:**
Edit `src/keymap.rs`:
- `get_default_keymap()` — update the `[[KeyAction; COL]; ROW]; NUM_LAYER]` array
- `get_default_encoder_map()` — update the `[[EncoderAction; NUM_ENCODER]; NUM_LAYER]` array

**Option B — Regenerate from macro expansion:**
If the keymap is complex, temporarily replace `src/central.rs` with the macro
version, expand, extract, then re-apply the joystick fix:

```bash
# 1. Save your hand-written central.rs
cp src/central.rs src/central.rs.bak

# 2. Temporarily revert to macro version
cat > src/central.rs << 'RUST'
#![no_main]
#![no_std]
use rmk::macros::rmk_central;
#[rmk_central]
mod keyboard_central {}
RUST

# 3. Expand to get generated keymap
RUST_MIN_STACK=67108864 cargo expand --bin central > /tmp/expanded.rs

# 4. Extract keymap functions (see scripts/extract_keymap.py)
python3 ../scripts/extract_keymap.py /tmp/expanded.rs > src/keymap.rs

# 5. Restore hand-written central.rs
cp src/central.rs.bak src/central.rs

# 6. Build
RUST_MIN_STACK=67108864 cargo build --release
cargo make uf2 --release
```

### Case 3: Big refactor (new peripherals, different matrix sizes, etc.)

If the structure changes significantly, it may be easier to regenerate from
scratch and re-apply the joystick fix:

```bash
cd ..
rm -rf rmk-local
rmkit create --keyboard-toml-path keyboard.toml --vial-json-path vial.json --target-dir rmk-local
# Then re-apply all patches (central.rs, keymap.rs, peripheral2, Makefile.toml)
```

## What the joystick fix does

In `src/central.rs`, the `EVENT_CHANNEL` consumer loop chains two processors:

```rust
loop {
    let event = EVENT_CHANNEL.receive().await;
    let mut current_event = event;

    // 1. Battery processor (passes through unhandled events)
    match battery_processor.process(current_event).await {
        ProcessResult::Stop => continue,
        ProcessResult::Continue(next_event) => current_event = next_event,
    }

    // 2. Joystick processor — converts Event::Joystick to HID mouse report
    match joystick_l.process(current_event).await {
        ProcessResult::Stop => continue,
        ProcessResult::Continue(_) => {}
    }
}
```

Both peripherals forward `Event::Joystick` via BLE split. The central receives
it, re-publishes to `EVENT_CHANNEL`, and the `JoystickProcessor` converts it
to a `MouseReport` sent to the host.
