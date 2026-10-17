# RoKi — RMK Firmware

Hand-written firmware for the RoKi split ergonomic keyboard, built on
[RMK](https://github.com/rmk-rs/rmk) **v0.9** with the native **dongle** feature.

## Topology

```
 ┌────────────┐  BLE split   ┌────────────┐  BLE (dongle slot)  ┌──────────┐  USB  ┌──────┐
 │ right half │ ───────────▶ │ left half  │ ──────────────────▶ │  dongle  │ ────▶ │ host │
 │ peripheral │              │  central   │ ──┐                 └──────────┘       └──────┘
 └────────────┘              └────────────┘   │ BLE profile 0 (direct, optional)
                                              └─────────────────────────────────▶ host
```

- The **left half is the keyboard**: it holds the keymap/storage, runs the split
  central, and talks to the host — either directly over a BLE profile, or through the dongle.
- The **dongle** is a pure relay: it bonds to exactly one keyboard and forwards HID
  reports and Vial traffic over USB. It has no keymap and stores only the bond.
- The **right half** is split peripheral 0.

## Hardware

- **Controller**: nice!nano v2 (nRF52840) ×3
- **Matrix**: 5×6 per half, row2col
- **Encoders**: 1 per half (P0.17 / P0.20)
- **Joysticks**: 1 per half (P0.31 X / P0.29 Y), 45° per-side rotation
- **Buzzer**: piezo on P0.06 per half, R2-D2 connect/disconnect sounds
- **Battery**: VDDH/5 via SAADC on each half

## File layout

| File | Role |
|------|------|
| `src/left.rs`    | Central: matrix, encoder 0, joystick (CCW 45°), battery, buzzer, USB + BLE transports |
| `src/right.rs`   | Peripheral 0: matrix, encoder 1, joystick (CW 45°), battery, buzzer |
| `src/dongle.rs`  | USB dongle: `rmk::dongle::Dongle` + `UsbTransport` sharing a `DongleRouter` |
| `src/devices.rs` | Shared `JoystickReader` (`#[input_device]`) and buzzer `#[processor]`s |
| `src/keymap.rs`  | Keymap, encoder map, Vial config (kept in sync with `keyboard.toml`) |
| `src/fmt.rs`     | Logging shim (`defmt` by default, `log` over USB with `--features usb_log`) + crash recorder |
| `keyboard.toml`  | Layout/keymap/pins; read at build time for RMK constants |
| `vial.json`      | Vial GUI layout descriptor |
| `vial_config.json` | Vial keymap export matching the firmware defaults (load it in Vial after a storage reset) |

## Prerequisites

RMK v0.9 on nRF needs an ARM C compiler (for trouble-host's P-256 crypto):

```bash
sudo apt install gcc-arm-none-eabi      # or download the ARM GNU toolchain and add bin/ to PATH
cargo install cargo-make flip-link cargo-binutils cargo-hex-to-uf2
```

## Build

```bash
RUST_MIN_STACK=67108864 cargo make uf2 --release
```

Outputs: `RoKi-left.uf2`, `RoKi-right.uf2`, `RoKi-dongle.uf2`.

## Flash

Double-tap reset on each nice!nano → drag the matching `.uf2` onto the `NICENANO` drive.

## Pairing the dongle

1. Flash all three boards and plug the dongle into the host. An unbonded dongle
   scans until a keyboard shows up.
2. On the keyboard, hold `MO(1)` and tap the **dongle key** (layer 1, right
   thumb cluster, outermost key). This is `SwitchToDongle` (`User(8)`): the
   keyboard switches to its dongle slot and starts seeking; the dongle pairs.
3. From then on both sides reconnect automatically. A bonded dongle opens one
   pairing window at power-on (30 s, `[rmk].dongle_pairing_window_secs`) and
   otherwise only reconnects its bonded keyboard.

**Hold the dongle key for 5 s** to clear the keyboard's dongle bond and seek a new dongle.

### Switching between dongle and direct BLE

Three BLE profiles plus the dongle slot. On layer 1 (`MO(1)`), thumb row:

| Side | Key | Keycode | Tap | Hold 5 s |
|---|---|---|---|---|
| left | 1st | `User(0)` | BLE profile 0 | clear its bond, re-pair |
| left | 2nd | `User(1)` | BLE profile 1 | clear its bond, re-pair |
| left | 3rd | `User(2)` | BLE profile 2 | clear its bond, re-pair |
| left | 5th | `User(5)` | clear bond of the *current* profile | — |
| right | last | `User(8)` | switch to dongle | clear dongle bond, seek new dongle |

Each slot keeps its own bond, so switching never requires re-pairing. USB
always wins when a cable is plugged in.

The split halves find each other on their own (no `ble_addr` needed).

**After upgrading from v0.8**: HID report IDs changed, so forget the keyboard
on any BLE host and re-pair.

## Debugging without a probe

Each half has a `usb_log` variant that replaces defmt with a USB CDC serial
port (logs at debug level, plus reset reason / last panic on boot):

```bash
cargo make uf2-left-usblog     # → RoKi-left-usblog.uf2
cargo make uf2-right-usblog    # → RoKi-right-usblog.uf2  (15 s boot countdown)
```

Open the new COM/`ttyACM` port at any baud rate. On Windows/PowerShell:

```powershell
$p = New-Object System.IO.Ports.SerialPort COM5,115200
$p.Open(); while ($true) { $p.ReadLine() }
```

Troubleshooting notes:

- **Right half never connects / no buzzer**: the left half logs
  `Start scanning peripherals` every 30 s but never `Found split peripheral`
  → the right half isn't running. Check that `sdc::Mem` is large enough for
  the controller config (`build_sdc` fails silently otherwise).
- **Red LED blinking, no USB device**: the board is in the bootloader. Two
  resets within ~500 ms (e.g. a crash-reset loop) look like a double-tap to
  the Adafruit bootloader.

## Calibrating joysticks

Tune the constants in `src/devices.rs` (`JoystickReader`):

```rust
const CENTER_X: i32 = 1900;  // resting ADC value
const CENTER_Y: i32 = 1900;
const SCALE: i32 = 128;      // sensitivity (larger = slower)
const DEAD_ZONE: i32 = 2;    // mouse units
```

## Updating the keymap

Edit both `keyboard.toml` (`[[keymap.layer]]`) and `src/keymap.rs` — the Rust
binaries use the hardcoded arrays; the TOML is the documented source of truth
and drives RMK's build-time constants. Keep `vial_config.json` in sync too.

Vial's stored keymap overrides the firmware default; the default is only
re-applied when the build hash changes (new build) or `clear_storage = true`.

## Default keymap

Layer 0: QWERTY, `MO(1)` on the outer bottom-row keys, encoders → macros 0–3
(rotate) / 8–9 (press).

Layer 1 (`MO(1)` held):

```
F1    F2    F3    F4    F5    F6        F7    F8    F9    F10   F11   F12
_     KP/   ¿?    Btn2  Btn3  Btn1      Btn1  Btn3  Btn2  _     _     _
_     '     _     ]     \     _         ←     ↓     ↑     →     _     _
_     _     _     ;     _     _         -     =     [     _     _     _
ble0  ble1  ble2  _     clrb  M10       M11   Del   _     _     _     dongle
```

## References

- [RMK Docs](https://rmk.rs/docs/)
- [RMK Dongle](https://rmk.rs/docs/features/dongle)
- [RMK v0.8 → v0.9 migration](https://rmk.rs/docs/migration/v08_v09)
