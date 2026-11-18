//! help_entry_json: one verb's object in the help's "verbs" array: name, group key, line and usage forms
//! (SPEC-cli 08, "The calls").
//! ZUT (rule 11): the help groups, find_help_entry, usage_forms and the JSON writers are pure and used directly.

use super::{find_help_entry, help_entry_json};

fn json_of(name: &str) -> String {
    let (group, entry) = find_help_entry(name).expect("the verb is in the help");
    help_entry_json(group, entry)
}

#[test]
fn guard_has_its_two_forms() {
    assert_eq!(
        json_of("guard"),
        r#"{"name":"guard","group":"landing","line":"the git hooks that keep integration and origin to Mortar","usage":["mr guard install --actor <name> [--key <operation key>] [--json]","mr guard status --actor <name> [--key <operation key>] [--json]"]}"#
    );
}

#[test]
fn a_quote_in_the_usage_is_escaped() {
    assert_eq!(
        json_of("save"),
        r#"{"name":"save","group":"agent","line":"keep your work as a commit","usage":["mr save \"<sentence>\" [--key <key>] [--json]"]}"#
    );
}

#[test]
fn an_apostrophe_in_the_line_is_kept() {
    assert_eq!(
        json_of("squash"),
        r#"{"name":"squash","group":"landing","line":"make a zone's squash","usage":["mr squash <zone> --actor <name> [--key <operation key>] [--json]"]}"#
    );
}
