//! RoKi Trackball keymap — keep in sync with `keyboard.toml` (`[[keymap.layer]]`).
#![allow(clippy::redundant_static_lifetimes)]

use rmk::types::action::KeyAction;
use rmk::{a, k, lt, user};

pub(crate) const COL: usize = 4;
pub(crate) const ROW: usize = 1;
pub(crate) const NUM_LAYER: usize = 2;

// With `ble_profiles_num = 3`: User(0..2) = profile. Hold a profile key 5 s
// to clear that slot's bond and re-pair.
const BLE0: KeyAction = user!(0);
const BLE1: KeyAction = user!(1);
const BLE2: KeyAction = user!(2);

#[rustfmt::skip]
pub const fn get_default_keymap() -> [[[KeyAction; COL]; ROW]; NUM_LAYER] {
    [
        // Layer 0: buttons; outer button is tap = back, hold = layer 1
        [[k!(MouseBtn1), k!(MouseBtn2), k!(MouseBtn3), lt!(1, MouseBtn4)]],
        // Layer 1: BLE profiles
        [[BLE0, BLE1, BLE2, a!(Transparent)]],
    ]
}

pub const KEYBOARD_DEVICE_CONFIG: rmk::config::DeviceConfig = rmk::config::DeviceConfig {
    vid: 0x4c4b,
    pid: 0x5254,
    manufacturer: "RoKi",
    product_name: "RoKi Trackball",
    serial_number: "vial:f64c2b3c:000002",
};

// Generated from vial.json by build.rs
include!(concat!(env!("OUT_DIR"), "/config_generated.rs"));

pub const VIAL_CONFIG: rmk::config::VialConfig = rmk::config::VialConfig {
    vial_keyboard_id: &VIAL_KEYBOARD_ID,
    vial_keyboard_def: &VIAL_KEYBOARD_DEF,
    unlock_keys: &[(0, 0), (0, 3)],
    insecure: false,
};
