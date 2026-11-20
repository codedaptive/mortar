//! item_json: one item as {"action","path","note"}, strings escaped, an absent path as "".

use std::path::PathBuf;

use super::{item_json, InstallItem};

fn item(action: &str, path: &str, note: &str) -> InstallItem {
    InstallItem { action: action.to_string(), path: PathBuf::from(path), note: note.to_string() }
}

#[test]
fn all_three_members_in_order() {
    assert_eq!(
        item_json(&item("linked", "/h/.local/bin/mr", "-> /h/.local/bin/mortar")),
        "{\"action\":\"linked\",\"path\":\"/h/.local/bin/mr\",\"note\":\"-> /h/.local/bin/mortar\"}"
    );
}

#[test]
fn an_empty_path_and_note_are_empty_strings() {
    assert_eq!(item_json(&item("gone", "", "")), "{\"action\":\"gone\",\"path\":\"\",\"note\":\"\"}");
}

#[test]
fn quotes_and_newlines_are_escaped() {
    assert_eq!(
        item_json(&item("left", "/h/\"q\"", "line\nbreak")),
        "{\"action\":\"left\",\"path\":\"/h/\\\"q\\\"\",\"note\":\"line\\nbreak\"}"
    );
}
