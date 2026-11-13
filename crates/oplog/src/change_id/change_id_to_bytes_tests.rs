//! to_bytes: the inverse of from_bytes, checked on a hand-written ID and as a round trip over every
//! value of every byte position.

use super::ChangeId;

#[test]
fn hand_written_id_gives_its_bytes() {
    let id = ChangeId::parse("zyxwvutsrqponmlkkkzzzzzzzzzzzzup").unwrap();
    assert_eq!(
        id.to_bytes(),
        [0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xff, 0, 0, 0, 0, 0, 0, 0x5a]
    );
}

/// Every byte value at every position, the other bytes filled with a pattern that differs per position.
fn inputs() -> impl Iterator<Item = [u8; 16]> {
    (0..16).flat_map(|position| {
        (0..=255u8).map(move |value| {
            let mut bytes = [0u8; 16];
            for (i, byte) in bytes.iter_mut().enumerate() {
                *byte = (i as u8).wrapping_mul(37).wrapping_add(position as u8);
            }
            bytes[position] = value;
            bytes
        })
    })
}

#[test]
fn from_bytes_then_to_bytes_returns_the_input() {
    for bytes in inputs() {
        assert_eq!(ChangeId::from_bytes(bytes).to_bytes(), bytes);
    }
}

#[test]
fn parse_of_from_bytes_text_equals_it() {
    for bytes in inputs() {
        let id = ChangeId::from_bytes(bytes);
        assert_eq!(ChangeId::parse(id.as_str()).unwrap(), id);
    }
}
