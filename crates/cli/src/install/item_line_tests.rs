//! item_line: action, then the path when there is one, then ": " and the note when there is one.

use std::path::PathBuf;

use super::{item_line, InstallItem};

fn item(action: &str, path: &str, note: &str) -> InstallItem {
    InstallItem { action: action.to_string(), path: PathBuf::from(path), note: note.to_string() }
}

#[test]
fn action_path_and_note() {
    assert_eq!(item_line(&item("linked", "/h/.local/bin/mr", "-> /h/.local/bin/mortar")), "linked /h/.local/bin/mr: -> /h/.local/bin/mortar");
}

#[test]
fn no_note_is_action_and_path() {
    assert_eq!(item_line(&item("removed", "/h/.local/bin/mortar", "")), "removed /h/.local/bin/mortar");
}

#[test]
fn no_path_is_action_and_note() {
    assert_eq!(item_line(&item("wiring skipped", "", "none picked")), "wiring skipped: none picked");
}

#[test]
fn neither_is_the_action_alone() {
    assert_eq!(item_line(&item("gone", "", "")), "gone");
}
