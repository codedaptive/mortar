//! find_help_entry: the row named exactly, with its group; None for any other word.
//! ZUT (rule 11): the help groups are pure data.

use super::find_help_entry;

#[test]
fn save_is_an_agent_verb() {
    let (group, entry) = find_help_entry("save").expect("save is in the help");
    assert_eq!(group.key, "agent");
    assert_eq!(entry.term, "save \"<sentence>\"");
    assert_eq!(entry.line, "keep your work as a commit");
}

#[test]
fn version_is_in_setup() {
    let (group, entry) = find_help_entry("--version").expect("--version is in the help");
    assert_eq!(group.key, "setup");
    assert_eq!(entry.name, "--version");
}

#[test]
fn restore_backup_is_a_landing_verb() {
    let (group, entry) = find_help_entry("restore-backup").expect("restore-backup is in the help");
    assert_eq!(group.key, "landing");
    assert_eq!(entry.name, "restore-backup");
}

#[test]
fn other_words_are_none() {
    for name in ["nosuch", "", "-h", "--help", "Save", "save "] {
        assert_eq!(find_help_entry(name), None, "{name:?}");
    }
}
