//! content_hash: FNV-1a 64 against its published test vectors, as 16 lowercase hex digits.

use super::content_hash;

#[test]
fn the_empty_input_is_the_offset_basis() {
    assert_eq!(content_hash(b""), "cbf29ce484222325");
}

#[test]
fn one_byte_matches_the_published_vector() {
    assert_eq!(content_hash(b"a"), "af63dc4c8601ec8c");
}

#[test]
fn several_bytes_match_the_published_vector() {
    assert_eq!(content_hash(b"foobar"), "85944171f73967e8");
}

#[test]
fn every_hash_is_16_lowercase_hex_digits() {
    for input in [&b""[..], b"a", b"foobar", b"\x00", b"\xff\xfe", b"a much longer input than the others"] {
        let hash = content_hash(input);
        assert_eq!(hash.len(), 16, "{hash}");
        assert!(hash.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')), "{hash}");
    }
}

#[test]
fn a_changed_byte_changes_the_hash() {
    assert_ne!(content_hash(b"foobar"), content_hash(b"foobaR"));
}
