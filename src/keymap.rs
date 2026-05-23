#![allow(clippy::redundant_static_lifetimes)]

pub(crate) const COL: usize = 12usize;
pub(crate) const ROW: usize = 5usize;
pub(crate) const NUM_LAYER: usize = 2usize;
pub(crate) const NUM_ENCODER: usize = 2usize;

use rmk::types::action::KeyAction;
use rmk::types::action::Action;
use rmk::types::action::EncoderAction;
use rmk::types::keycode::KeyCode;
use rmk::types::keycode::HidKeyCode;

pub const fn get_default_keymap() -> [[[KeyAction; COL]; ROW]; NUM_LAYER] {
    [
        [
            [KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Escape))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Kc1))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Kc2))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Kc3))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Kc4))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Kc5))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Kc6))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Kc7))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Kc8))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Kc9))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Kc0))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Escape)))],
            [KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Tab))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Q))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::W))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::E))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::R))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::T))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Y))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::U))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::I))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::O))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::P))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Grave)))],
            [KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::LShift))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::A))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::S))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::D))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::F))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::G))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::H))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::J))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::K))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::L))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Slash))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::RShift)))],
            [KeyAction::Single(Action::LayerOn(1u8)), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Z))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::X))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::C))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::V))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::B))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::N))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::M))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Comma))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Dot))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Backslash))), KeyAction::Single(Action::LayerOn(1u8))],
            [KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::LCtrl))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::LGui))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::LAlt))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Space))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Enter))), KeyAction::No, KeyAction::No, KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Backspace))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Space))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::RAlt))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::RGui))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::RCtrl)))],
        ],
        [
            [KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::F1))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::F2))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::F3))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::F4))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::F5))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::F6))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::F7))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::F8))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::F9))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::F10))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::F11))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::F12)))],
            [KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::MouseLeft))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::MouseDown))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::MouseUp))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::MouseRight))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::MediaPlayPause))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::AudioVolUp)))],
            [KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::LShift))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Quote))), KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Left))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Down))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Up))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Right))), KeyAction::Transparent, KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::AudioVolDown)))],
            [KeyAction::Single(Action::LayerOn(1u8)), KeyAction::Transparent, KeyAction::Transparent, KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Semicolon))), KeyAction::Transparent, KeyAction::Transparent, KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Minus))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Equal))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::LeftBracket))), KeyAction::Transparent, KeyAction::Transparent, KeyAction::Single(Action::LayerOn(1u8))],
            [KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::LCtrl))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::LGui))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::LAlt))), KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::Transparent, KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Delete))), KeyAction::Transparent, KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::RAlt))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::RGui))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::RCtrl)))],
        ],
    ]
}

pub const fn get_default_encoder_map() -> [[EncoderAction; NUM_ENCODER]; NUM_LAYER] {
    [
        [
            EncoderAction::new(KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::AudioVolDown))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::AudioVolUp)))),
            EncoderAction::new(KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::AudioVolDown))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::AudioVolUp)))),
        ],
        [
            EncoderAction::new(KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Down))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Up)))),
            EncoderAction::new(KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Down))), KeyAction::Single(Action::Key(KeyCode::Hid(HidKeyCode::Up)))),
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

pub const VIAL_CONFIG: ::rmk::config::VialConfig = ::rmk::config::VialConfig {
    vial_keyboard_id: &[0xB9, 0xBC, 0x09, 0xB2, 0x9D, 0x37, 0x4C, 0xEA],
    vial_keyboard_def: &[
        253u8, 55u8, 122u8, 88u8, 90u8, 0u8, 0u8, 4u8, 230u8, 214u8, 180u8, 70u8, 2u8, 0u8,
        33u8, 1u8, 22u8, 0u8, 0u8, 0u8, 116u8, 47u8, 229u8, 163u8, 224u8, 4u8, 60u8, 1u8,
        147u8, 93u8, 0u8, 61u8, 136u8, 137u8, 198u8, 84u8, 54u8, 195u8, 23u8, 79u8, 228u8,
        229u8, 149u8, 118u8, 171u8, 224u8, 33u8, 82u8, 27u8, 250u8, 237u8, 242u8, 19u8, 69u8,
        222u8, 42u8, 250u8, 139u8, 23u8, 88u8, 84u8, 175u8, 95u8, 40u8, 177u8, 114u8, 212u8,
        83u8, 13u8, 82u8, 171u8, 106u8, 76u8, 237u8, 127u8, 52u8, 250u8, 75u8, 115u8, 197u8,
        13u8, 171u8, 235u8, 188u8, 189u8, 116u8, 103u8, 68u8, 123u8, 110u8, 146u8, 47u8,
        235u8, 183u8, 59u8, 77u8, 201u8, 10u8, 130u8, 24u8, 193u8, 208u8, 18u8, 25u8, 183u8,
        206u8, 155u8, 45u8, 136u8, 42u8, 202u8, 135u8, 106u8, 30u8, 68u8, 148u8, 123u8,
        200u8, 172u8, 102u8, 178u8, 180u8, 246u8, 178u8, 16u8, 14u8, 28u8, 99u8, 219u8,
        103u8, 152u8, 14u8, 134u8, 244u8, 129u8, 145u8, 152u8, 233u8, 105u8, 158u8, 63u8,
        95u8, 132u8, 82u8, 121u8, 67u8, 240u8, 204u8, 128u8, 71u8, 157u8, 245u8, 219u8, 88u8,
        252u8, 48u8, 117u8, 255u8, 81u8, 255u8, 181u8, 4u8, 96u8, 182u8, 253u8, 3u8, 14u8,
        117u8, 255u8, 121u8, 238u8, 23u8, 252u8, 85u8, 108u8, 134u8, 5u8, 203u8, 183u8, 61u8,
        132u8, 190u8, 198u8, 11u8, 14u8, 121u8, 176u8, 253u8, 137u8, 18u8, 99u8, 195u8,
        175u8, 2u8, 186u8, 217u8, 222u8, 58u8, 42u8, 223u8, 41u8, 98u8, 253u8, 197u8, 161u8,
        11u8, 51u8, 18u8, 117u8, 235u8, 69u8, 80u8, 118u8, 37u8, 122u8, 26u8, 186u8, 117u8,
        237u8, 154u8, 156u8, 204u8, 204u8, 146u8, 165u8, 8u8, 62u8, 119u8, 178u8, 16u8,
        242u8, 41u8, 72u8, 59u8, 188u8, 192u8, 72u8, 76u8, 168u8, 112u8, 165u8, 213u8, 107u8,
        172u8, 250u8, 133u8, 130u8, 165u8, 251u8, 235u8, 113u8, 73u8, 22u8, 18u8, 184u8,
        56u8, 192u8, 226u8, 185u8, 34u8, 235u8, 69u8, 73u8, 188u8, 32u8, 173u8, 39u8, 21u8,
        35u8, 109u8, 142u8, 54u8, 72u8, 215u8, 79u8, 152u8, 206u8, 90u8, 214u8, 204u8, 250u8,
        176u8, 112u8, 49u8, 110u8, 35u8, 182u8, 137u8, 142u8, 247u8, 231u8, 217u8, 204u8,
        72u8, 210u8, 253u8, 31u8, 115u8, 177u8, 65u8, 204u8, 215u8, 5u8, 89u8, 224u8, 24u8,
        223u8, 95u8, 162u8, 46u8, 50u8, 178u8, 119u8, 54u8, 118u8, 102u8, 197u8, 228u8,
        121u8, 217u8, 49u8, 17u8, 180u8, 200u8, 7u8, 142u8, 45u8, 24u8, 233u8, 59u8, 36u8,
        245u8, 30u8, 130u8, 144u8, 218u8, 128u8, 42u8, 73u8, 183u8, 40u8, 93u8, 169u8, 6u8,
        8u8, 255u8, 241u8, 227u8, 44u8, 162u8, 49u8, 223u8, 138u8, 193u8, 55u8, 74u8, 199u8,
        52u8, 33u8, 10u8, 38u8, 111u8, 204u8, 233u8, 106u8, 18u8, 114u8, 50u8, 65u8, 237u8,
        219u8, 68u8, 231u8, 153u8, 219u8, 174u8, 49u8, 21u8, 70u8, 72u8, 246u8, 39u8, 184u8,
        139u8, 88u8, 108u8, 202u8, 252u8, 149u8, 121u8, 255u8, 123u8, 170u8, 155u8, 138u8,
        253u8, 40u8, 30u8, 96u8, 104u8, 176u8, 145u8, 91u8, 22u8, 9u8, 111u8, 95u8, 248u8,
        110u8, 172u8, 60u8, 11u8, 99u8, 185u8, 130u8, 248u8, 210u8, 3u8, 207u8, 147u8, 136u8,
        153u8, 160u8, 50u8, 249u8, 75u8, 217u8, 161u8, 235u8, 94u8, 155u8, 118u8, 199u8,
        106u8, 0u8, 0u8, 224u8, 127u8, 24u8, 146u8, 20u8, 126u8, 93u8, 176u8, 0u8, 1u8,
        175u8, 3u8, 189u8, 8u8, 0u8, 0u8, 201u8, 202u8, 180u8, 141u8, 177u8, 196u8, 103u8,
        251u8, 2u8, 0u8, 0u8, 0u8, 0u8, 4u8, 89u8, 90u8,
    ],
    unlock_keys: &[],
};
