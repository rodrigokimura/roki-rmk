//! RoKi keymap — keep in sync with `keyboard.toml` (`[[keymap.layer]]`).
#![allow(clippy::redundant_static_lifetimes)]

use rmk::types::action::{EncoderAction, KeyAction};
use rmk::types::modifier::ModifierCombination;
use rmk::{a, encoder, k, macros, mo, user, wm};

pub(crate) const COL: usize = 12;
pub(crate) const ROW: usize = 6;
pub(crate) const NUM_LAYER: usize = 3;
pub(crate) const NUM_ENCODER: usize = 2;

// With `ble_profiles_num = 3`: User(0..2) = profile, User(5) = clear current
// bond, User(8) = SwitchToDongle. Hold any profile/dongle key 5 s to clear
// that slot's bond and re-pair.
const BLE0: KeyAction = user!(0);
const BLE1: KeyAction = user!(1);
const BLE2: KeyAction = user!(2);
const BLE_NEXT: KeyAction = user!(3);
const BLE_PREV: KeyAction = user!(4);
const CLEAR_BOND: KeyAction = user!(5);
const CLEAR_SPLIT: KeyAction = user!(6);
const DONGLE: KeyAction = user!(8);

#[rustfmt::skip]
pub const fn get_default_keymap() -> [[[KeyAction; COL]; ROW]; NUM_LAYER] {
    [
        // Layer 0
        [
            [k!(Escape), k!(Kc1), k!(Kc2), k!(Kc3), k!(Kc4), k!(Kc5),      k!(Kc6), k!(Kc7), k!(Kc8), k!(Kc9), k!(Kc0), mo!(2)],
            [k!(Tab),    k!(Q),   k!(W),   k!(E),   k!(R),   k!(T),        k!(Y),   k!(U),   k!(I),   k!(O),   k!(P),   k!(Grave)],
            [k!(LShift), k!(A),   k!(S),   k!(D),   k!(F),   k!(G),        k!(H),   k!(J),   k!(K),   k!(L),   k!(Slash), k!(RShift)],
            [mo!(1),     k!(Z),   k!(X),   k!(C),   k!(V),   k!(B),        k!(N),   k!(M),   k!(Comma), k!(Dot), k!(Backslash), mo!(1)],
            [k!(LCtrl),  k!(LGui), k!(LAlt), k!(Space), k!(Enter), k!(AudioMute),   k!(AudioMute), k!(Backspace), k!(Space), k!(RAlt), k!(RGui), k!(RCtrl)],
            // Direct-pin keys: (5, 0) = left P0.22, (5, 6) = right P0.22.
            [k!(MouseBtn1), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent),
                k!(MouseBtn1), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
        ],
        // Layer 1
        [
            [k!(F1), k!(F2), k!(F3), k!(F4), k!(F5), k!(F6),      k!(F7), k!(F8), k!(F9), k!(F10), k!(F11), k!(F12)],
            [a!(Transparent), k!(KpSlash), wm!(International1, ModifierCombination::LSHIFT), k!(MouseBtn2), k!(MouseBtn3), k!(MouseBtn1),
                k!(MouseBtn1), k!(MouseBtn3), k!(MouseBtn2), a!(Transparent), a!(Transparent), a!(Transparent)],
            [a!(Transparent), k!(Quote), a!(Transparent), k!(RightBracket), k!(Backslash), a!(Transparent),
                k!(Left), k!(Down), k!(Up), k!(Right), a!(Transparent), a!(Transparent)],
            [a!(Transparent), a!(Transparent), a!(Transparent), k!(Semicolon), a!(Transparent), a!(Transparent),
                k!(Minus), k!(Equal), k!(LeftBracket), a!(Transparent), a!(Transparent), a!(Transparent)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent),
                a!(Transparent), k!(Delete), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent),
                a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
        ],
        // Layer 3
        [
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent),
                a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent),
                a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent),
                a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent),
                a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
            [BLE0, BLE1, BLE2, DONGLE, BLE_PREV, a!(Transparent),
                a!(Transparent), BLE_NEXT, a!(Transparent), a!(Transparent), CLEAR_SPLIT, CLEAR_BOND],
            [a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent),
                a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent), a!(Transparent)],
        ],
    ]
}

pub const fn get_default_encoder_map() -> [[EncoderAction; NUM_ENCODER]; NUM_LAYER] {
    [
        [
            encoder!(k!(BrightnessUp), k!(BrightnessDown)),
            encoder!(k!(AudioVolUp), k!(AudioVolDown)),
        ],
        [
            encoder!(k!(MouseWheelUp), k!(MouseWheelDown)),
            encoder!(k!(MouseWheelUp), k!(MouseWheelDown)),
        ],
        [
            encoder!(a!(Transparent), a!(Transparent)),
            encoder!(a!(Transparent), a!(Transparent)),
        ],
    ]
}

pub const KEYBOARD_DEVICE_CONFIG: rmk::config::DeviceConfig = rmk::config::DeviceConfig {
    vid: 0x4c4b,
    pid: 0x524b,
    manufacturer: "RoKi",
    product_name: "RoKi Keyboard",
    serial_number: "vial:f64c2b3c:000001",
};

// Generated from vial.json by build.rs
include!(concat!(env!("OUT_DIR"), "/config_generated.rs"));

pub const VIAL_CONFIG: rmk::config::VialConfig = rmk::config::VialConfig {
    vial_keyboard_id: &VIAL_KEYBOARD_ID,
    vial_keyboard_def: &VIAL_KEYBOARD_DEF,
    unlock_keys: &[(0, 0), (0, 11)],
    insecure: false,
};
