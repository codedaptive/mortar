use crate::count_chars::count_chars;

#[test]
fn empty_is_zero() {
    assert_eq!(count_chars(b""), 0);
}

#[test]
fn ascii_counts_each_byte() {
    assert_eq!(count_chars(b"hello, world\n"), 13);
}

#[test]
fn a_multibyte_character_counts_once() {
    assert_eq!(count_chars("é".as_bytes()), 1);
    assert_eq!(count_chars("✓".as_bytes()), 1);
    assert_eq!(count_chars("日本語".as_bytes()), 3);
    assert_eq!(count_chars("😀".as_bytes()), 1);
    assert_eq!(count_chars("héllo wörld ✓ 日本語\n".as_bytes()), 18);
}

#[test]
fn a_lone_lead_byte_counts_one() {
    assert_eq!(count_chars(b"\xff"), 1);
}

#[test]
fn a_lone_continuation_byte_counts_none() {
    assert_eq!(count_chars(b"\x80"), 0);
}

#[test]
fn a_truncated_sequence_counts_its_lead_byte() {
    // The first two of the three bytes of 日.
    assert_eq!(count_chars(b"\xe6\x97"), 1);
    assert_eq!(count_chars(b"a\xe6\x97b"), 3);
}

#[test]
fn any_split_of_a_buffer_sums_to_the_whole() {
    // a, é, ✓, 日, 本, 語, 😀, a lone 0xFF and b count one each; the lone 0x80 counts none.
    let mut bytes = "aé✓日本語😀".as_bytes().to_vec();
    bytes.extend_from_slice(b"\xff\x80b");
    assert_eq!(count_chars(&bytes), 9);
    for split in 0..=bytes.len() {
        let (head, tail) = bytes.split_at(split);
        assert_eq!(count_chars(head) + count_chars(tail), 9, "split at {split}");
    }
}
