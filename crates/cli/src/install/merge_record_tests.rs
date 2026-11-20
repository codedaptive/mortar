//! merge_record: an earlier record kept in order, items at the same path replaced in place, new ones appended.

use std::path::PathBuf;

use super::{merge_record, RecordItem};

fn file(path: &str, hash: &str) -> RecordItem {
    RecordItem::File { path: PathBuf::from(path), hash: hash.to_string() }
}

fn dir(path: &str) -> RecordItem {
    RecordItem::Dir { path: PathBuf::from(path) }
}

fn link(path: &str, target: &str) -> RecordItem {
    RecordItem::Link { path: PathBuf::from(path), target: PathBuf::from(target) }
}

#[test]
fn an_empty_old_record_gives_the_new_items() {
    let new = [dir("/h/.local/bin"), file("/h/.local/bin/mortar", "1")];
    assert_eq!(merge_record(&[], &new), vec![dir("/h/.local/bin"), file("/h/.local/bin/mortar", "1")]);
}

#[test]
fn nothing_new_keeps_the_old_record() {
    let old = [dir("/h/.local/bin"), file("/h/.local/bin/mortar", "1")];
    assert_eq!(merge_record(&old, &[]), vec![dir("/h/.local/bin"), file("/h/.local/bin/mortar", "1")]);
}

#[test]
fn an_item_at_the_same_path_is_replaced_in_place_and_others_appended() {
    let old = [dir("/h/.local/bin"), file("/h/.local/bin/mortar", "1"), link("/h/.local/bin/mr", "/h/.local/bin/mortar")];
    let new = [file("/h/.claude/skills/mortar/SKILL.md", "9"), file("/h/.local/bin/mortar", "2")];
    assert_eq!(
        merge_record(&old, &new),
        vec![
            dir("/h/.local/bin"),
            file("/h/.local/bin/mortar", "2"),
            link("/h/.local/bin/mr", "/h/.local/bin/mortar"),
            file("/h/.claude/skills/mortar/SKILL.md", "9"),
        ]
    );
}

#[test]
fn a_new_item_of_another_kind_at_an_old_path_replaces_it() {
    let old = [file("/h/.local/bin/mr", "1")];
    let new = [link("/h/.local/bin/mr", "/opt/bin/mortar")];
    assert_eq!(merge_record(&old, &new), vec![link("/h/.local/bin/mr", "/opt/bin/mortar")]);
}
