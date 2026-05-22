"""
Extract get_default_keymap() and get_default_encoder_map() from
a cargo-expand output and clean them up for use in src/keymap.rs.

Usage:
    python3 extract_keymap.py /tmp/expanded.rs > src/keymap.rs
"""
import re
import sys

def main(path):
    with open(path) as f:
        content = f.read()

    # Extract constants at module level
    consts = {}
    for m in re.finditer(r'pub\(crate\) const (\w+): usize = (\d+)usize;', content):
        consts[m.group(1)] = m.group(2)

    # Extract get_default_keymap
    keymap_match = re.search(
        r'(pub const fn get_default_keymap\(\).*?)(?=pub const fn get_default_encoder_map|fn ble_addr|#\[embassy_executor::main\])',
        content, re.DOTALL
    )
    if not keymap_match:
        print("ERROR: get_default_keymap not found", file=sys.stderr)
        sys.exit(1)
    keymap_fn = keymap_match.group(1)

    # Extract get_default_encoder_map
    enc_match = re.search(
        r'(pub const fn get_default_encoder_map\(\).*?)(?=fn ble_addr|#\[embassy_executor::main\]|use ::embassy_nrf::bind_interrupts)',
        content, re.DOTALL
    )
    if not enc_match:
        print("ERROR: get_default_encoder_map not found", file=sys.stderr)
        sys.exit(1)
    enc_fn = enc_match.group(1)

    # Extract device config and VIAL config (they may need manual verification)
    device_match = re.search(
        r'(const KEYBOARD_DEVICE_CONFIG: ::rmk::config::DeviceConfig = .*?};)',
        content, re.DOTALL
    )
    vial_match = re.search(
        r'(const VIAL_CONFIG: ::rmk::config::VialConfig = .*?};)',
        content, re.DOTALL
    )

    # Clean up overly verbose enum paths
    def simplify(text):
        # Replace ::rmk::types::action::KeyAction with KeyAction, etc.
        text = re.sub(r'::rmk::types::action::KeyAction', 'KeyAction', text)
        text = re.sub(r'::rmk::types::action::Action', 'Action', text)
        text = re.sub(r'::rmk::types::action::EncoderAction', 'EncoderAction', text)
        text = re.sub(r'::rmk::types::keycode::KeyCode::', 'KeyCode::', text)
        text = re.sub(r'::rmk::config::', '::rmk::config::', text)  # keep as-is
        # Shorten obvious paths in keymap
        text = text.replace('::rmk::types::action::', '')
        return text

    out = []
    out.append("#![allow(clippy::redundant_static_lifetimes)]\n")
    for name in ['COL', 'ROW', 'NUM_LAYER', 'NUM_ENCODER']:
        if name in consts:
            out.append(f"pub(crate) const {name}: usize = {consts[name]}usize;\n")
    out.append("")
    out.append("use rmk::types::action::KeyAction;\n")
    out.append("use rmk::types::action::Action;\n")
    out.append("use rmk::types::encoder::EncoderAction;\n")
    out.append("use rmk::types::keycode::KeyCode;\n\n")

    out.append(simplify(keymap_fn))
    out.append("\n\n")
    out.append(simplify(enc_fn))

    if device_match:
        out.append("\n\n")
        out.append("pub ")
        out.append(device_match.group(1))
    if vial_match:
        out.append("\n\n")
        out.append("pub ")
        # VIAL_CONFIG contains huge byte arrays; keep them as-is
        out.append(vial_match.group(1))

    print("".join(out))

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python3 extract_keymap.py <expanded.rs>", file=sys.stderr)
        sys.exit(1)
    main(sys.argv[1])
