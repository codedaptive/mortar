//! step_record: the record item each kind of step leaves.

use std::path::PathBuf;

use super::{step_record, InstallStep, RecordItem};

#[test]
fn a_placed_binary_is_a_file_with_the_binary_hash() {
    let step = InstallStep::PlaceBinary { from: PathBuf::from("/src/mortar"), to: PathBuf::from("/h/.local/bin/mortar") };
    assert_eq!(
        step_record(&step, "0123456789abcdef"),
        RecordItem::File { path: PathBuf::from("/h/.local/bin/mortar"), hash: "0123456789abcdef".to_string() }
    );
}

#[test]
fn a_link_is_a_link_with_its_target() {
    let step = InstallStep::Link { at: PathBuf::from("/h/.local/bin/mr"), target: PathBuf::from("/h/.local/bin/mortar") };
    assert_eq!(
        step_record(&step, "0123456789abcdef"),
        RecordItem::Link { path: PathBuf::from("/h/.local/bin/mr"), target: PathBuf::from("/h/.local/bin/mortar") }
    );
}

#[test]
fn a_skill_is_a_file_with_its_text_hash_not_the_binary_hash() {
    // FNV-1a 64 of "foobar", the published vector.
    let step = InstallStep::WriteSkill { path: PathBuf::from("/h/.claude/skills/mortar/SKILL.md"), text: "foobar".to_string() };
    assert_eq!(
        step_record(&step, "0123456789abcdef"),
        RecordItem::File {
            path: PathBuf::from("/h/.claude/skills/mortar/SKILL.md"),
            hash: "85944171f73967e8".to_string()
        }
    );
}
