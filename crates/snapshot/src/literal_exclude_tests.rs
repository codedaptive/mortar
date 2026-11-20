use crate::literal_exclude::literal_exclude;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

#[test]
fn a_plain_path_gets_a_colon_and_an_escaped_first_character() {
    assert_eq!(literal_exclude(Path::new("src/a.rs")), Some(":\\src/a.rs".to_string()));
}

#[test]
fn backslash_and_glob_characters_are_escaped() {
    assert_eq!(literal_exclude(Path::new("a*b?[c]\\d")), Some(":\\a\\*b\\?\\[c]\\\\d".to_string()));
}

#[test]
fn a_glob_first_character_gets_one_backslash() {
    assert_eq!(literal_exclude(Path::new("*x")), Some(":\\*x".to_string()));
}

#[test]
fn a_path_starting_with_a_colon_keeps_it_after_the_escape() {
    assert_eq!(literal_exclude(Path::new(":x")), Some(":\\:x".to_string()));
}

#[test]
fn a_space_is_left_as_it_is() {
    assert_eq!(literal_exclude(Path::new("dir/with space.bin")), Some(":\\dir/with space.bin".to_string()));
}

#[test]
fn a_non_ascii_first_character_is_escaped_whole() {
    assert_eq!(literal_exclude(Path::new("é.bin")), Some(":\\é.bin".to_string()));
}

#[test]
fn a_path_that_is_not_utf8_gives_none() {
    assert_eq!(literal_exclude(Path::new(OsStr::from_bytes(b"a\xffb"))), None);
}

#[test]
fn an_empty_path_gives_none() {
    assert_eq!(literal_exclude(Path::new("")), None);
}
