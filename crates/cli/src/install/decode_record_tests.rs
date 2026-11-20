//! decode_record: hand-written records read back, the round trip with encode_record over many items, and each
//! refusal naming the item index.

use std::path::PathBuf;

use super::{decode_record, encode_record, RecordItem};

fn file(path: &str, hash: &str) -> RecordItem {
    RecordItem::File { path: PathBuf::from(path), hash: hash.to_string() }
}

fn link(path: &str, target: &str) -> RecordItem {
    RecordItem::Link { path: PathBuf::from(path), target: PathBuf::from(target) }
}

fn dir(path: &str) -> RecordItem {
    RecordItem::Dir { path: PathBuf::from(path) }
}

#[test]
fn a_hand_written_record_with_spaces_and_any_key_order_is_read() {
    let text = "{ \"items\" : [\n  {\"path\": \"/h/.local/bin/mortar\", \"kind\": \"file\", \"hash\": \"cbf29ce484222325\"},\n  \
                {\"target\": \"/h/.local/bin/mortar\", \"kind\": \"link\", \"path\": \"/h/.local/bin/mr\"},\n  \
                {\"kind\": \"dir\", \"path\": \"/h/.local/bin\"}\n] }\n";
    assert_eq!(
        decode_record(text),
        Ok(vec![
            file("/h/.local/bin/mortar", "cbf29ce484222325"),
            link("/h/.local/bin/mr", "/h/.local/bin/mortar"),
            dir("/h/.local/bin"),
        ])
    );
}

#[test]
fn escapes_are_read_back() {
    assert_eq!(
        decode_record("{\"items\":[{\"kind\":\"dir\",\"path\":\"/h/a\\\"b\\\\c\\nd\\u0001\"}]}"),
        Ok(vec![dir("/h/a\"b\\c\nd\u{1}")])
    );
}

#[test]
fn an_empty_record_has_no_items() {
    assert_eq!(decode_record("{\"items\":[]}\n"), Ok(Vec::new()));
}

#[test]
fn encode_then_decode_returns_the_items_over_many_records() {
    let names = ["/h/.local/bin/mortar", "/h/sp ace", "/h/quo\"te", "/h/back\\slash", "/h/new\nline", "/h/tab\t",
        "/h/caf\u{e9}/\u{1f600}", "/h/ctl\u{1}\u{1f}", "", "/"];
    let mut items = Vec::new();
    for (index, name) in names.iter().enumerate() {
        items.push(file(name, &format!("{index:016x}")));
        items.push(link(name, names[(index + 1) % names.len()]));
        items.push(dir(name));
    }
    for count in 0..=items.len() {
        let slice = &items[..count];
        let text = encode_record(slice).expect("every path is UTF-8");
        assert_eq!(decode_record(&text).as_deref(), Ok(slice), "{text}");
    }
}

fn refused(text: &str, words: &[&str]) {
    let reason = decode_record(text).expect_err(text);
    for word in words {
        assert!(reason.contains(word), "{reason:?} should name {word:?}");
    }
}

#[test]
fn a_missing_kind_is_refused_naming_the_item() {
    refused("{\"items\":[{\"kind\":\"dir\",\"path\":\"/a\"},{\"path\":\"/b\"}]}", &["item 1", "\"kind\""]);
}

#[test]
fn an_unknown_kind_is_refused_naming_the_item() {
    refused("{\"items\":[{\"kind\":\"socket\",\"path\":\"/a\"}]}", &["item 0", "\"socket\""]);
}

#[test]
fn a_missing_key_is_refused_naming_the_item_and_key() {
    refused("{\"items\":[{\"kind\":\"dir\",\"path\":\"/a\"},{\"kind\":\"file\",\"path\":\"/b\"}]}", &["item 1", "\"hash\""]);
    refused("{\"items\":[{\"kind\":\"link\",\"path\":\"/b\"}]}", &["item 0", "\"target\""]);
    refused("{\"items\":[{\"kind\":\"dir\"}]}", &["item 0", "\"path\""]);
}

#[test]
fn an_extra_key_is_refused_naming_the_item_and_key() {
    refused("{\"items\":[{\"kind\":\"dir\",\"path\":\"/a\",\"hash\":\"x\"}]}", &["item 0", "\"hash\""]);
    refused(
        "{\"items\":[{\"kind\":\"dir\",\"path\":\"/a\"},{\"kind\":\"dir\",\"path\":\"/b\"},\
         {\"kind\":\"link\",\"path\":\"/c\",\"target\":\"/d\",\"mode\":\"x\"}]}",
        &["item 2", "\"mode\""],
    );
}

#[test]
fn a_key_twice_is_refused_naming_the_item_and_key() {
    refused("{\"items\":[{\"kind\":\"dir\",\"path\":\"/a\",\"path\":\"/a\"}]}", &["item 0", "\"path\"", "twice"]);
    refused("{\"items\":[{\"kind\":\"dir\",\"kind\":\"dir\",\"path\":\"/a\"}]}", &["item 0", "\"kind\"", "twice"]);
}

#[test]
fn a_value_that_is_not_a_string_is_refused_naming_the_item() {
    refused("{\"items\":[{\"kind\":\"dir\",\"path\":7}]}", &["item 0", "\"path\""]);
    refused("{\"items\":[{\"kind\":\"dir\",\"path\":\"/a\"},\"/b\"]}", &["item 1"]);
}

#[test]
fn text_that_is_not_a_record_is_refused() {
    for text in ["", "{\"items\":[", "[]", "{\"items\":{}}", "{\"things\":[]}", "{\"items\":[],\"items\":[]}",
        "{\"items\":[],\"more\":1}", "{\"items\":[]} trailing"] {
        assert!(decode_record(text).is_err(), "{text:?}");
    }
}
