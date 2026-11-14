//! attempt_number: canonical decimal from 1 to u32::MAX accepted; everything else None.

use super::attempt_number;

#[test]
fn canonical_decimals_are_read() {
    assert_eq!(attempt_number("1"), Some(1));
    assert_eq!(attempt_number("10"), Some(10));
    assert_eq!(attempt_number("4294967295"), Some(4294967295));
}

#[test]
fn every_other_spelling_is_none() {
    for text in ["", "0", "01", "+1", "-1", " 1", "1 ", "4294967296", "x"] {
        assert_eq!(attempt_number(text), None, "{text:?}");
    }
}
