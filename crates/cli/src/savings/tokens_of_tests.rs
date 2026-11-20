//! tokens_of: characters / 4, rounded half up.

use super::tokens_of;

#[test]
fn the_contract_values() {
    let cases = [(2_161_007, 540_252), (54_368, 13_592), (0, 0), (1, 0), (2, 1), (6, 2), (5, 1), (7, 2)];
    for (chars, tokens) in cases {
        assert_eq!(tokens_of(chars), tokens, "{chars}");
    }
}

#[test]
fn the_largest_count_does_not_overflow() {
    assert_eq!(tokens_of(u64::MAX), 4_611_686_018_427_387_904);
}
