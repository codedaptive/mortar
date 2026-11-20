//! encode_record: the one-line JSON form of install's record, hand-written; a path that is not UTF-8 is Err.

use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

use super::{encode_record, RecordItem};

#[test]
fn no_items_is_an_empty_array_and_a_newline() {
    assert_eq!(encode_record(&[]), Ok("{\"items\":[]}\n".to_string()));
}

#[test]
fn each_kind_has_its_keys_in_order() {
    let items = [
        RecordItem::File { path: PathBuf::from("/h/.local/bin/mortar"), hash: "0123456789abcdef".to_string() },
        RecordItem::Link { path: PathBuf::from("/h/.local/bin/mr"), target: PathBuf::from("/h/.local/bin/mortar") },
        RecordItem::Dir { path: PathBuf::from("/h/.local/bin") },
    ];
    assert_eq!(
        encode_record(&items),
        Ok(concat!(
            "{\"items\":[",
            "{\"kind\":\"file\",\"path\":\"/h/.local/bin/mortar\",\"hash\":\"0123456789abcdef\"},",
            "{\"kind\":\"link\",\"path\":\"/h/.local/bin/mr\",\"target\":\"/h/.local/bin/mortar\"},",
            "{\"kind\":\"dir\",\"path\":\"/h/.local/bin\"}",
            "]}\n"
        )
        .to_string())
    );
}

#[test]
fn quotes_backslashes_and_newlines_are_escaped_onto_one_line() {
    let items = [RecordItem::Dir { path: PathBuf::from("/h/a\"b\\c\nd") }];
    assert_eq!(encode_record(&items), Ok("{\"items\":[{\"kind\":\"dir\",\"path\":\"/h/a\\\"b\\\\c\\nd\"}]}\n".to_string()));
}

#[test]
fn a_path_that_is_not_utf8_is_returned_as_the_error() {
    let bad = PathBuf::from(OsStr::from_bytes(b"/h/\xff"));
    let items = [
        RecordItem::Dir { path: PathBuf::from("/h/fine") },
        RecordItem::Link { path: PathBuf::from("/h/mr"), target: bad.clone() },
    ];
    assert_eq!(encode_record(&items), Err(bad));
}
