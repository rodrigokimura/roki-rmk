# RoKi — RMK Cloud Compilation

> Build RMK firmware for RoKi via GitHub Actions — no local Rust toolchain needed.

## Why Cloud Compilation?

The local Rust + `nrf-softdevice` + `bindgen` stack is complex and fragile (libclang, git dependencies, feature flags, etc.). RMK provides a [GitHub project template](https://github.com/HaoboGu/rmk-project-template) that compiles your firmware in the cloud using GitHub Actions.

## How It Works

1. You define your keyboard in `keyboard.toml` (pins, matrix, keymap, split config)
2. You define the Vial layout in `vial.json`
3. Push to GitHub → GitHub Action runs `rmkit create` to generate Rust code from your config
4. The action compiles both `central.uf2` (left half) and `peripheral.uf2` (right half)
5. Download the artifacts and flash via UF2 bootloader

## Project Structure

```
roki-rmk-cloud/
├── .github/workflows/build.yml    # GitHub Action workflow
├── keyboard.toml                    # RoKi hardware + keymap config
├── vial.json                       # Vial GUI layout descriptor
└── README.md                        # This file
```

## Quick Start

### 1. Create a GitHub repository

Option A — Use the RMK template (easiest):
1. Go to https://github.com/HaoboGu/rmk-project-template
2. Click **"Use this template"** → **"Create a new repository"**
3. Name it `roki-rmk` (or whatever)

Option B — Use this folder directly:
1. Create a new empty repo on GitHub named `roki-rmk`
2. Clone it locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/roki-rmk.git
   cd roki-rmk
   ```

### 2. Copy these files into the repo

If using Option B, copy the files from this folder:
```bash
cp /mnt/c/Users/kimur/projects/roki/roki-rmk-cloud/keyboard.toml .
cp /mnt/c/Users/kimur/projects/roki/roki-rmk-cloud/vial.json .
cp -r /mnt/c/Users/kimur/projects/roki/roki-rmk-cloud/.github .
```

### 3. Push to GitHub

```bash
git add .
git commit -m "feat: add RoKi keyboard config"
git push
```

### 4. Wait for the build

1. Go to your GitHub repo → **Actions** tab
2. You should see a workflow running
3. Wait ~5-10 minutes for compilation
4. Go to the latest workflow run → **Summary**
5. Download the artifacts:
   - `firmware_uf2` → contains `central.uf2` and `peripheral.uf2`

### 5. Flash to your nice!nano boards

1. **Left half** — double-tap the reset button → drag `central.uf2` onto `NRF52BOOT`
2. **Right half** — double-tap the reset button → drag `peripheral.uf2` onto `NRF52BOOT`
3. Pair the left half with your computer via Bluetooth

## `keyboard.toml` Reference

| Section | What it does |
|---------|--------------|
| `[keyboard]` | Name, VID/PID, chip (`nrf52840`) |
| `[layout]` | 10×6 keymap, 2 layers, encoder map |
| `[ble]` | Enable BLE wireless |
| `[storage]` | Enable flash storage (required for BLE split) |
| `[split]` | Split keyboard with BLE connection |
| `[split.central]` | Left half: 5×6 matrix, rows 0-4 |
| `[[split.peripheral]]` | Right half: 5×6 matrix, rows 5-9 |
| `[[input_device.encoder]]` | Rotary encoder pins (P0.17, P0.20) |

### Pin Mapping (nice!nano v2)

| Function | nRF52840 Pin |
|----------|--------------|
| Matrix Row 1 | P0.24 |
| Matrix Row 2 | P1.00 |
| Matrix Row 3 | P0.11 |
| Matrix Row 4 | P1.04 |
| Matrix Row 5 | P1.06 |
| Matrix Col 1 | P0.09 |
| Matrix Col 2 | P0.10 |
| Matrix Col 3 | P1.11 |
| Matrix Col 4 | P1.13 |
| Matrix Col 5 | P1.15 |
| Matrix Col 6 | P0.02 |
| Encoder A | P0.17 |
| Encoder B | P0.20 |

## Customizing the Keymap

Edit `keyboard.toml` → `[layout]` → `keymap` array. Each layer is a 10×6 grid.

Keycodes follow RMK's string format:
- Letters/numbers: `A`, `B`, `Q`, `Kc1`, `Kc2`, ...
- Modifiers: `LCtrl`, `LShift`, `LAlt`, `LGui`, `RCtrl`, `RShift`, `RAlt`, `RGui`
- Special: `Escape`, `Tab`, `Space`, `Enter`, `Backspace`, `Delete`, `Grave`, `Comma`, `Dot`, `Slash`, `Backslash`, `Semicolon`, `Quote`, `LeftBracket`, `RightBracket`, `Minus`, `Equal`
- Arrows: `Up`, `Down`, `Left`, `Right`
- Media: `AudioVolUp`, `AudioVolDown`, `MediaPlayPause`
- Mouse: `MouseUp`, `MouseDown`, `MouseLeft`, `MouseRight`
- Layer: `MO(1)`, `TG(1)`, `TO(1)`, `OSL(1)`
- No-op: `No` or `_`

After editing, commit and push. GitHub Actions will rebuild automatically.

## What This Doesn't Include (Yet)

| Feature | Status | Notes |
|---------|--------|-------|
| Analog thumbstick → mouse | ❌ Not supported by `keyboard.toml` | Requires custom Rust code (Phase 3) |
| Piezo buzzer | ❌ Not supported by `keyboard.toml` | Requires custom Rust code (Phase 4) |
| Thumbstick calibration | ❌ Not supported | Requires custom code + NVMC storage |
| Per-half keymap mirroring | ⚠️ Experimental | `keyboard.toml` uses a single unified keymap |

These features would need the `use_rust` API approach. The cloud compilation path gets you a working matrix + BLE + layers + encoders much faster.

## Troubleshooting

### Build fails on GitHub Actions

1. Check the **Actions** log for the exact error
2. Common issues:
   - Invalid pin names (must be `P0_00` format, not `P0.00`)
   - Keymap row/col count doesn't match `layout.rows`/`layout.cols`
   - `matrix_map` is required if using `[[layer]]` format (we use the simpler `keymap` array instead)

### No Bluetooth device appears after flashing

1. Make sure you flashed `central.uf2` to the **left** half
2. Clear existing bonds on your computer's Bluetooth settings
3. Re-flash both halves (sometimes the first boot writes garbage to storage)
4. Try pressing the `MO(1)` key (layer 1) + a number key to switch BLE profiles

### Encoders don't work

The `keyboard.toml` encoder config applies globally. If only one encoder works, the split peripheral encoder support in RMK might need additional configuration. Open an issue on [RMK](https://github.com/HaoboGu/rmk/issues) with your `keyboard.toml`.

## Links

- [RMK Configuration Docs](https://rmk.rs/docs/configuration.html)
- [RMK Split Keyboard Docs](https://rmk.rs/docs/features/split_keyboard.html)
- [RMK Cloud Compilation Docs](https://rmk.rs/docs/user_guide/create_firmware/cloud_compilation.html)
- [RoKi Hardware Pinout](../kicad/roki/)

---

*This is an experimental cloud-compilation path. For maximum control (custom analog thumbstick, buzzer, etc.), the local `roki-firmware-rmk/` Rust project is still the long-term target.*
