//! plan_uninstall: on install's map under the test HOME `/h`, unchanged files and links are removed, gone ones named
//! gone, changed ones left; folders still folders go to `dirs`, deepest first, with no item; gone folders have no
//! item; a folder path holding anything else is left and named. Entries off the map are skipped and named, whatever
//! their state.

use std::path::{Path, PathBuf};

use super::{plan_uninstall, InstallItem, ItemState, RecordItem, UninstallPlan};

fn p(path: &str) -> PathBuf {
    PathBuf::from(path)
}

fn item(action: &str, path: &str, note: &str) -> InstallItem {
    InstallItem { action: action.to_string(), path: p(path), note: note.to_string() }
}

#[test]
fn an_empty_record_plans_nothing() {
    assert_eq!(plan_uninstall(&[], &[], Path::new("/h")), UninstallPlan { remove: vec![], dirs: vec![], items: vec![] });
}

#[test]
fn a_file_with_the_same_hash_and_a_link_with_the_same_target_are_removed() {
    let record = [
        RecordItem::File { path: p("/h/.local/bin/mortar"), hash: "aaaa".to_string() },
        RecordItem::Link { path: p("/h/.local/bin/mr"), target: p("/h/.local/bin/mortar") },
    ];
    let states = [ItemState::File { hash: "aaaa".to_string() }, ItemState::Link { target: p("/h/.local/bin/mortar") }];
    assert_eq!(
        plan_uninstall(&record, &states, Path::new("/h")),
        UninstallPlan {
            remove: vec![p("/h/.local/bin/mortar"), p("/h/.local/bin/mr")],
            dirs: vec![],
            items: vec![item("removed", "/h/.local/bin/mortar", ""), item("removed", "/h/.local/bin/mr", "")],
        }
    );
}

#[test]
fn changed_files_are_left_and_named() {
    let record = [
        RecordItem::File { path: p("/h/.claude/skills/mortar/SKILL.md"), hash: "aaaa".to_string() },
        RecordItem::File { path: p("/h/.local/bin/mortar"), hash: "aaaa".to_string() },
        RecordItem::File { path: p("/h/.agents/skills/mortar/SKILL.md"), hash: "aaaa".to_string() },
        RecordItem::File { path: p("/h/.cursor/skills/mortar/SKILL.md"), hash: "aaaa".to_string() },
    ];
    let states = [
        ItemState::File { hash: "bbbb".to_string() },
        ItemState::Link { target: p("/h/.local/bin/mortar") },
        ItemState::Dir,
        ItemState::Other,
    ];
    let left = [
        "/h/.claude/skills/mortar/SKILL.md",
        "/h/.local/bin/mortar",
        "/h/.agents/skills/mortar/SKILL.md",
        "/h/.cursor/skills/mortar/SKILL.md",
    ]
    .map(|path| item("left", path, "changed since install"));
    assert_eq!(
        plan_uninstall(&record, &states, Path::new("/h")),
        UninstallPlan { remove: vec![], dirs: vec![], items: left.to_vec() }
    );
}

#[test]
fn a_changed_link_is_left_and_named() {
    let record = [RecordItem::Link { path: p("/h/.local/bin/mr"), target: p("/h/.local/bin/mortar") }];
    for state in [
        ItemState::Link { target: p("/elsewhere") },
        ItemState::File { hash: "aaaa".to_string() },
        ItemState::Dir,
        ItemState::Other,
    ] {
        assert_eq!(
            plan_uninstall(&record, std::slice::from_ref(&state), Path::new("/h")),
            UninstallPlan {
                remove: vec![],
                dirs: vec![],
                items: vec![item("left", "/h/.local/bin/mr", "changed since install")],
            },
            "{state:?}"
        );
    }
}

#[test]
fn gone_items_are_named_gone_and_not_removed() {
    let record = [
        RecordItem::File { path: p("/h/.local/bin/mortar"), hash: "aaaa".to_string() },
        RecordItem::Link { path: p("/h/.local/bin/mr"), target: p("/h/.local/bin/mortar") },
    ];
    assert_eq!(
        plan_uninstall(&record, &[ItemState::Gone, ItemState::Gone], Path::new("/h")),
        UninstallPlan {
            remove: vec![],
            dirs: vec![],
            items: vec![item("gone", "/h/.local/bin/mortar", ""), item("gone", "/h/.local/bin/mr", "")],
        }
    );
}

#[test]
fn folders_still_folders_go_deepest_first_with_no_item() {
    let record = [
        RecordItem::Dir { path: p("/h/.local") },
        RecordItem::Dir { path: p("/h/.local/bin") },
        RecordItem::Dir { path: p("/h/.agents") },
        RecordItem::Dir { path: p("/h/.local/state/mortar") },
        RecordItem::Dir { path: p("/h/.agents/skills/mortar") },
        RecordItem::Dir { path: p("/h/.local/state") },
        RecordItem::Dir { path: p("/h/.agents/skills") },
    ];
    let states = vec![ItemState::Dir; record.len()];
    assert_eq!(
        plan_uninstall(&record, &states, Path::new("/h")),
        UninstallPlan {
            remove: vec![],
            dirs: vec![
                p("/h/.local/state/mortar"),
                p("/h/.agents/skills/mortar"),
                p("/h/.local/state"),
                p("/h/.local/bin"),
                p("/h/.agents/skills"),
                p("/h/.local"),
                p("/h/.agents"),
            ],
            items: vec![],
        }
    );
}

#[test]
fn a_gone_folder_has_no_item_and_is_not_tried() {
    let record = [RecordItem::Dir { path: p("/h/.agents") }, RecordItem::Dir { path: p("/h/.local/bin") }];
    assert_eq!(
        plan_uninstall(&record, &[ItemState::Gone, ItemState::Dir], Path::new("/h")),
        UninstallPlan { remove: vec![], dirs: vec![p("/h/.local/bin")], items: vec![] }
    );
}

#[test]
fn a_folder_that_became_a_file_a_link_or_other_is_left_and_named_not_tried() {
    let record = [
        RecordItem::Dir { path: p("/h/.claude") },
        RecordItem::Dir { path: p("/h/.claude/skills") },
        RecordItem::Dir { path: p("/h/.cursor/skills/mortar") },
        RecordItem::Dir { path: p("/h/.local/state") },
    ];
    let states = [
        ItemState::File { hash: "aaaa".to_string() },
        ItemState::Link { target: p("/elsewhere") },
        ItemState::Other,
        ItemState::Dir,
    ];
    assert_eq!(
        plan_uninstall(&record, &states, Path::new("/h")),
        UninstallPlan {
            remove: vec![],
            dirs: vec![p("/h/.local/state")],
            items: vec![
                item("left", "/h/.claude", "no longer a folder"),
                item("left", "/h/.claude/skills", "no longer a folder"),
                item("left", "/h/.cursor/skills/mortar", "no longer a folder"),
            ],
        }
    );
}

#[test]
fn a_mixed_record_keeps_record_order_for_items() {
    let record = [
        RecordItem::Dir { path: p("/h/.local/bin") },
        RecordItem::File { path: p("/h/.local/bin/mortar"), hash: "aaaa".to_string() },
        RecordItem::Link { path: p("/h/.local/bin/mr"), target: p("/h/.local/bin/mortar") },
        RecordItem::File { path: p("/h/.claude/skills/mortar/SKILL.md"), hash: "cccc".to_string() },
    ];
    let states = [
        ItemState::Dir,
        ItemState::File { hash: "aaaa".to_string() },
        ItemState::Gone,
        ItemState::File { hash: "dddd".to_string() },
    ];
    assert_eq!(
        plan_uninstall(&record, &states, Path::new("/h")),
        UninstallPlan {
            remove: vec![p("/h/.local/bin/mortar")],
            dirs: vec![p("/h/.local/bin")],
            items: vec![
                item("removed", "/h/.local/bin/mortar", ""),
                item("gone", "/h/.local/bin/mr", ""),
                item("left", "/h/.claude/skills/mortar/SKILL.md", "changed since install"),
            ],
        }
    );
}

#[test]
fn entries_off_the_map_are_skipped_and_named_whatever_their_state() {
    let record = [
        RecordItem::File { path: p("/h/notes.txt"), hash: "aaaa".to_string() },
        RecordItem::Link { path: p("/h/.claude/skills/mortar/SKILL.md"), target: p("/h/elsewhere") },
        RecordItem::File { path: p("/h/.local/bin/mr"), hash: "aaaa".to_string() },
        RecordItem::Dir { path: p("/h/keep") },
        RecordItem::Dir { path: p("/h") },
        RecordItem::File { path: p("/h/gone.txt"), hash: "aaaa".to_string() },
        RecordItem::Dir { path: p("/h/.local/share") },
    ];
    let states = [
        ItemState::File { hash: "aaaa".to_string() },
        ItemState::Link { target: p("/h/elsewhere") },
        ItemState::File { hash: "aaaa".to_string() },
        ItemState::Dir,
        ItemState::Dir,
        ItemState::Gone,
        ItemState::Other,
    ];
    let skipped = [
        "/h/notes.txt",
        "/h/.claude/skills/mortar/SKILL.md",
        "/h/.local/bin/mr",
        "/h/keep",
        "/h",
        "/h/gone.txt",
        "/h/.local/share",
    ]
    .map(|path| item("skipped", path, "not a path install places"));
    assert_eq!(
        plan_uninstall(&record, &states, Path::new("/h")),
        UninstallPlan { remove: vec![], dirs: vec![], items: skipped.to_vec() }
    );
}

#[test]
fn an_off_map_entry_does_not_stop_the_on_map_ones() {
    let record = [
        RecordItem::File { path: p("/h/notes.txt"), hash: "aaaa".to_string() },
        RecordItem::File { path: p("/h/.local/bin/mortar"), hash: "aaaa".to_string() },
        RecordItem::Dir { path: p("/h/keep") },
        RecordItem::Dir { path: p("/h/.local/bin") },
    ];
    let states = [
        ItemState::File { hash: "aaaa".to_string() },
        ItemState::File { hash: "aaaa".to_string() },
        ItemState::Dir,
        ItemState::Dir,
    ];
    assert_eq!(
        plan_uninstall(&record, &states, Path::new("/h")),
        UninstallPlan {
            remove: vec![p("/h/.local/bin/mortar")],
            dirs: vec![p("/h/.local/bin")],
            items: vec![
                item("skipped", "/h/notes.txt", "not a path install places"),
                item("removed", "/h/.local/bin/mortar", ""),
                item("skipped", "/h/keep", "not a path install places"),
            ],
        }
    );
}
