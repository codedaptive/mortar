//! hash_key: "<prefix>-<16 hex>" of FNV-1a 64 over the NUL-ended fields.

use super::hash_key;

#[test]
fn no_fields_give_the_fnv_offset_basis() {
    assert_eq!(hash_key("p", &[]), "p-cbf29ce484222325");
}

#[test]
fn the_nul_ends_each_field() {
    assert_ne!(hash_key("p", &[b"ab"]), hash_key("p", &[b"a", b"b"]));
}

#[test]
fn the_prefix_is_the_text_before_the_dash() {
    let key = hash_key("restore", &[b"x"]);
    let (prefix, hex) = key.split_once('-').expect("a dash");
    assert_eq!(prefix, "restore");
    assert_eq!(hex.len(), 16);
    assert!(hex.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
}
