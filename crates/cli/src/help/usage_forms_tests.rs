//! usage_forms: a verb's usage one line per form of its "( a | b )" group, unchanged without one; --version's
//! line; nothing for an unknown name.
//! Unit test: args/find_verb, the verb table and split_usage_forms are pure and used directly.

use super::usage_forms;

#[test]
fn folder_has_six_forms() {
    assert_eq!(
        usage_forms("folder"),
        vec![
            "mr folder zone <zone> <path> [--start <commit>] --actor <name> [--key <operation key>] [--json]",
            "mr folder request <zone> <request> <path> --paths <path>... --actor <name> [--key <operation key>] \
             [--json]",
            "mr folder remove <path> --actor <name> [--key <operation key>] [--json]",
            "mr folder attempts <n> <dir> --actor <name> [--key <operation key>] [--json]",
            "mr folder keep <k> --actor <name> [--key <operation key>] [--json]",
            "mr folder drop --actor <name> [--key <operation key>] [--json]",
        ]
    );
}

#[test]
fn guard_has_two_forms() {
    assert_eq!(
        usage_forms("guard"),
        vec![
            "mr guard install --actor <name> [--key <operation key>] [--json]",
            "mr guard status --actor <name> [--key <operation key>] [--json]",
        ]
    );
}

#[test]
fn guard_hook_has_two_forms_with_nothing_after() {
    assert_eq!(
        usage_forms("guard-hook"),
        vec!["mortar guard-hook reference-transaction <state>", "mortar guard-hook pre-push <remote> <url>"]
    );
}

#[test]
fn status_is_one_line() {
    assert_eq!(usage_forms("status"), vec!["mr status [--json]"]);
}

#[test]
fn help_is_one_line() {
    assert_eq!(usage_forms("help"), vec!["mr help [<verb>] [--json]"]);
}

#[test]
fn version_has_its_line() {
    assert_eq!(usage_forms("--version"), vec!["mr --version [--json]"]);
}

#[test]
fn an_unknown_name_has_no_lines() {
    for name in ["nosuch", "", "-h"] {
        assert_eq!(usage_forms(name), Vec::<String>::new(), "{name:?}");
    }
}
