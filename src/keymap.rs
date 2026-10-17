#![allow(clippy::redundant_static_lifetimes)]

pub(crate) const COL: usize = 12usize;
pub(crate) const ROW: usize = 5usize;
pub(crate) const NUM_LAYER: usize = 2usize;
pub(crate) const NUM_ENCODER: usize = 2usize;

use rmk::types::action::KeyAction;
use rmk::types::action::Action;
use rmk::types::action::EncoderAction;
use rmk::types::keycode::KeyCode;
use rmk::types::modifier::ModifierCombination;

pub const fn get_default_keymap() -> [[[KeyAction; COL]; ROW]; NUM_LAYER] {
    [
        [
            [KeyAction::Single(Action::Key(KeyCode::Escape)), KeyAction::Single(Action::Key(KeyCode::Kc1)), KeyAction::Single(Action::Key(KeyCode::Kc2)), KeyAction::Single(Action::Key(KeyCode::Kc3)), KeyAction::Single(Action::Key(KeyCode::Kc4)), KeyAction::Single(Action::Key(KeyCode::Kc5)), KeyAction::Single(Action::Key(KeyCode::Kc6)), KeyAction::Single(Action::Key(KeyCode::Kc7)), KeyAction::Single(Action::Key(KeyCode::Kc8)), KeyAction::Single(Action::Key(KeyCode::Kc9)), KeyAction::Single(Action::Key(KeyCode::Kc0)), KeyAction::Single(Action::Key(KeyCode::Escape))],
            [KeyAction::Single(Action::Key(KeyCode::Tab)), KeyAction::Single(Action::Key(KeyCode::Q)), KeyAction::Single(Action::Key(KeyCode::W)), KeyAction::Single(Action::Key(KeyCode::E)), KeyAction::Single(Action::Key(KeyCode::R)), KeyAction::Single(Action::Key(KeyCode::T)), KeyAction::Single(Action::Key(KeyCode::Y)), KeyAction::Single(Action::Key(KeyCode::U)), KeyAction::Single(Action::Key(KeyCode::I)), KeyAction::Single(Action::Key(KeyCode::O)), KeyAction::Single(Action::Key(KeyCode::P)), KeyAction::Single(Action::Key(KeyCode::Grave))],
            [KeyAction::Single(Action::Key(KeyCode::LShift)), KeyAction::Single(Action::Key(KeyCode::A)), KeyAction::Single(Action::Key(KeyCode::S)), KeyAction::Single(Action::Key(KeyCode::D)), KeyAction::Single(Action::Key(KeyCode::F)), KeyAction::Single(Action::Key(KeyCode::G)), KeyAction::Single(Action::Key(KeyCode::H)), KeyAction::Single(Action::Key(KeyCode::J)), KeyAction::Single(Action::Key(KeyCode::K)), KeyAction::Single(Action::Key(KeyCode::L)), KeyAction::Single(Action::Key(KeyCode::Slash)), KeyAction::Single(Action::Key(KeyCode::RShift))],
            [KeyAction::Single(Action::LayerOn(1u8)), KeyAction::Single(Action::Key(KeyCode::Z)), KeyAction::Single(Action::Key(KeyCode::X)), KeyAction::Single(Action::Key(KeyCode::C)), KeyAction::Single(Action::Key(KeyCode::V)), KeyAction::Single(Action::Key(KeyCode::B)), KeyAction::Single(Action::Key(KeyCode::N)), KeyAction::Single(Action::Key(KeyCode::M)), KeyAction::Single(Action::Key(KeyCode::Comma)), KeyAction::Single(Action::Key(KeyCode::Dot)), KeyAction::Single(Action::Key(KeyCode::Backslash)), KeyAction::Single(Action::LayerOn(1u8))],
            [KeyAction::Single(Action::Key(KeyCode::LCtrl)), KeyAction::Single(Action::Key(KeyCode::LGui)), KeyAction::Single(Action::Key(KeyCode::LAlt)), KeyAction::Single(Action::Key(KeyCode::Space)), KeyAction::Single(Action::Key(KeyCode::Enter)), KeyAction::Single(Action::Key(KeyCode::Macro8)), KeyAction::Single(Action::Key(KeyCode::Macro9)), KeyAction::Single(Action::Key(KeyCode::Backspace)), KeyAction::Single(Action::Key(KeyCode::Space)), KeyAction::Single(Action::Key(KeyCode::RAlt)), KeyAction::Single(Action::Key(KeyCode::RGui)), KeyAction::Single(Action::Key(KeyCode::RCtrl))],
        ],
        [
            [KeyAction::Single(Action::Key(KeyCode::F1)), KeyAction::Single(Action::Key(KeyCode::F2)), KeyAction::Single(Action::Key(KeyCode::F3)), KeyAction::Single(Action::Key(KeyCode::F4)), KeyAction::Single(Action::Key(KeyCode::F5)), KeyAction::Single(Action::Key(KeyCode::F6)), KeyAction::Single(Action::Key(KeyCode::F7)), KeyAction::Single(Action::Key(KeyCode::F8)), KeyAction::Single(Action::Key(KeyCode::F9)), KeyAction::Single(Action::Key(KeyCode::F10)), KeyAction::Single(Action::Key(KeyCode::F11)), KeyAction::Single(Action::Key(KeyCode::F12))],
            [KeyAction::Transparent, KeyAction::Single(Action::Key(KeyCode::KpSlash)), KeyAction::Single(Action::KeyWithModifier(KeyCode::International1, ModifierCombination::LSHIFT)), KeyAction::Single(Action::Key(KeyCode::MouseBtn2)), KeyAction::Single(Action::Key(KeyCode::MouseBtn3)), KeyAction::Single(Action::Key(KeyCode::MouseBtn1)), KeyAction::Single(Action::Key(KeyCode::MouseBtn1)), KeyAction::Single(Action::Key(KeyCode::MouseBtn3)), KeyAction::Single(Action::Key(KeyCode::MouseBtn2)), KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent],
            [KeyAction::Transparent, KeyAction::Single(Action::Key(KeyCode::Quote)), KeyAction::Transparent, KeyAction::Single(Action::Key(KeyCode::RightBracket)), KeyAction::Single(Action::Key(KeyCode::Backslash)), KeyAction::Transparent, KeyAction::Single(Action::Key(KeyCode::Left)), KeyAction::Single(Action::Key(KeyCode::Down)), KeyAction::Single(Action::Key(KeyCode::Up)), KeyAction::Single(Action::Key(KeyCode::Right)), KeyAction::Transparent, KeyAction::Transparent],
            [KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::Single(Action::Key(KeyCode::Semicolon)), KeyAction::Transparent, KeyAction::Transparent, KeyAction::Single(Action::Key(KeyCode::Minus)), KeyAction::Single(Action::Key(KeyCode::Equal)), KeyAction::Single(Action::Key(KeyCode::LeftBracket)), KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent],
            [KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::Single(Action::Key(KeyCode::Delete)), KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent],
        ],
    ]
}

pub const fn get_default_encoder_map() -> [[EncoderAction; NUM_ENCODER]; NUM_LAYER] {
    [
        [
            EncoderAction::new(KeyAction::Single(Action::Key(KeyCode::Macro0)), KeyAction::Single(Action::Key(KeyCode::Macro1))),
            EncoderAction::new(KeyAction::Single(Action::Key(KeyCode::Macro2)), KeyAction::Single(Action::Key(KeyCode::Macro3))),
        ],
        [
            EncoderAction::new(KeyAction::Single(Action::Key(KeyCode::Macro4)), KeyAction::Single(Action::Key(KeyCode::Macro5))),
            EncoderAction::new(KeyAction::Single(Action::Key(KeyCode::Macro6)), KeyAction::Single(Action::Key(KeyCode::Macro7))),
        ],
    ]
}

pub const KEYBOARD_DEVICE_CONFIG: ::rmk::config::DeviceConfig = ::rmk::config::DeviceConfig {
    vid: 19531u16,
    pid: 21067u16,
    manufacturer: "RoKi",
    product_name: "RoKi Keyboard",
    serial_number: "vial:f64c2b3c:000001",
};

// Generated from vial.json by build.rs
include!(concat!(env!("OUT_DIR"), "/config_generated.rs"));

pub const VIAL_CONFIG: ::rmk::config::VialConfig = ::rmk::config::VialConfig {
    vial_keyboard_id: &VIAL_KEYBOARD_ID,
    vial_keyboard_def: &VIAL_KEYBOARD_DEF,
    unlock_keys: &[(0, 0), (0, 11)],
};
