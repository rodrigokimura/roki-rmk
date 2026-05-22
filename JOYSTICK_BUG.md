# Known Issue: Joysticks on BLE Split Peripherals

**Status:** Workaround implemented in `rmk-local/`  
**Affected hardware:** nRF52 BLE split keyboards with joysticks on peripheral halves  
**Date documented:** 2026-05-22

---

## Symptom

Joysticks configured in `keyboard.toml` under `[[split.peripheral.input_device.joystick]]` do not move the mouse cursor. The matrix, encoders, layers, and everything else works fine.

## Root cause

RMK's code generation for split peripherals places the `JoystickProcessor` on the **peripheral** device, but the peripheral has no HID transport to the host. Only the **central** (dongle) can send HID reports to the host via USB or BLE.

### Event flow that fails

```
Peripheral half:
  NrfAdc reads joystick pins
    → generates Event::Joystick
      → JoystickProcessor converts to MouseReport
        → KEYBOARD_REPORT_CHANNEL  ← DROPPED — no transport on peripheral
      → SplitPeripheral forwards SplitMessage::Event to central  ← works

Central/dongle:
  PeripheralManager receives SplitMessage::Event
    → re-publishes Event::Joystick on central's EVENT_CHANNEL
      → NO JoystickProcessor on central to consume it  ← BUG
        → MouseReport never generated
```

### Why encoders work

Encoders generate `KeyboardEvent`s (virtual key presses). Those are forwarded through split messages and processed by the central's normal keymap engine. Joysticks generate `Event::Joystick`, which need a `JoystickProcessor` to turn them into `MouseReport`s — and that processor only exists on the peripheral.

---

## Workaround: Local compilation (`rmk-local/`)

Cloud compilation (`rmkit create`) does not give enough control. The workaround uses **local compilation** with a hand-written `central.rs` that explicitly includes `JoystickProcessor` on the central.

### What was changed

| File | Change |
|------|--------|
| `rmk-local/src/central.rs` | Replaced `#[rmk_central]` macro with explicit task setup; added `JoystickProcessor` that reads from `EVENT_CHANNEL` alongside `BatteryProcessor` |
| `rmk-local/src/keymap.rs` | Extracted keymap, encoder map, and VIAL config from macro-generated code |
| `rmk-local/src/peripheral2.rs` | Added second peripheral binary (`id = 1`) |
| `rmk-local/Cargo.toml` | Added `[[bin]] peripheral2` |
| `rmk-local/Makefile.toml` | Added `objcopy-peripheral2` and `uf2-peripheral2` tasks |

### Key fix in `central.rs`

The central now runs a combined processor loop that chains `BatteryProcessor` → `JoystickProcessor` on the same `EVENT_CHANNEL`:

```rust
loop {
    let event = EVENT_CHANNEL.receive().await;
    let mut current_event = event;

    // Battery processor (passes through unhandled events)
    match battery_processor.process(current_event).await {
        ProcessResult::Stop => continue,
        ProcessResult::Continue(next_event) => current_event = next_event,
    }

    // Joystick processor (THE FIX)
    match joystick_l.process(current_event).await {
        ProcessResult::Stop => continue,
        ProcessResult::Continue(_) => {}
    }
}
```

This ensures `Event::Joystick` forwarded from both peripherals is converted to HID mouse reports on the dongle.

### Build instructions

```bash
cd rmk-local
RUST_MIN_STACK=67108864 cargo build --release
cargo make uf2 --release
```

Outputs:
- `RoKi-central.uf2` → flash to dongle
- `RoKi-peripheral.uf2` → flash to left half
- `RoKi-peripheral2.uf2` → flash to right half

### Limitations

- Both joysticks share a single `JoystickProcessor` on the central (same transform/bias/resolution). Since both halves use identical calibration, this works correctly.
- The `EVENT_CHANNEL` is single-consumer, so processors must be chained sequentially rather than run in parallel tasks.

---

## Upstream fix (recommended long-term)

File an issue on https://github.com/HaoboGu/rmk/issues. The proper fix would be for `rmk-macro` to detect joysticks on split peripherals and auto-generate `JoystickProcessor` initializers on the central's task list.

---

## References

- RMK joystick docs: https://rmk.rs/docs/configuration/input_device/joystick.html
- `Event::Joystick` forwarding in split driver: `rmk/src/split/driver.rs:process_peripheral_message()`
- `JoystickProcessor` codegen on peripheral: `rmk-macro/src/codegen/input_device/adc.rs`
- `InputProcessor` trait: `rmk/src/input_device/mod.rs`
