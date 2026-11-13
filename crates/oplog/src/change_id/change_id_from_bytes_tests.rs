//! from_bytes: two chars per byte, high nibble first, nibble n as b'z' - n (0 is 'z', 15 is 'k').

use super::ChangeId;

#[test]
fn all_zero_bytes_give_32_z() {
    assert_eq!(ChangeId::from_bytes([0u8; 16]).as_str(), "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz");
}

#[test]
fn all_ff_bytes_give_32_k() {
    assert_eq!(ChangeId::from_bytes([0xff; 16]).as_str(), "kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
}

#[test]
fn every_nibble_value_has_its_letter() {
    let bytes = [0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0, 0, 0, 0, 0, 0, 0, 0];
    assert_eq!(ChangeId::from_bytes(bytes).as_str(), "zyxwvutsrqponmlkzzzzzzzzzzzzzzzz");
}

#[test]
fn high_nibble_comes_first() {
    let mut low = [0u8; 16];
    low[0] = 0x0f;
    let mut high = [0u8; 16];
    high[0] = 0xf0;
    assert_eq!(ChangeId::from_bytes(low).as_str(), "zkzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz");
    assert_eq!(ChangeId::from_bytes(high).as_str(), "kzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz");
}

#[test]
fn last_byte_lands_in_last_two_chars() {
    let mut bytes = [0u8; 16];
    bytes[15] = 0x5a;
    assert_eq!(ChangeId::from_bytes(bytes).as_str(), "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzup");
}
