//! verb_help_lines: a verb's usage one line per form, then its row's line (SPEC-cli 08, "A verb's help").
//! ZUT (rule 11): the help groups, find_help_entry, usage_forms and the verb table are pure and used directly.

use super::{find_help_entry, verb_help_lines};

fn help_of(name: &str) -> Vec<String> {
    let (_, entry) = find_help_entry(name).expect("the verb is in the help");
    verb_help_lines(entry)
}

#[test]
fn status_is_its_usage_and_its_line() {
    assert_eq!(
        help_of("status"),
        vec![
            "mr status [--json]",
            "where you are, what you changed, whether your zone moved, anything conflicted, and the next step",
        ]
    );
}

#[test]
fn folder_is_six_forms_and_its_line() {
    assert_eq!(
        help_of("folder"),
        vec![
            "mr folder zone <zone> <path> [--start <commit>] --actor <name> [--key <operation key>] [--json]",
            "mr folder request <zone> <request> <path> --paths <path>... --actor <name> [--key <operation key>] \
             [--json]",
            "mr folder remove <path> --actor <name> [--key <operation key>] [--json]",
            "mr folder attempts <n> <dir> --actor <name> [--key <operation key>] [--json]",
            "mr folder keep <k> --actor <name> [--key <operation key>] [--json]",
            "mr folder drop --actor <name> [--key <operation key>] [--json]",
            "make and remove zone, request and attempt folders",
        ]
    );
}

#[test]
fn version_is_its_usage_and_its_line() {
    assert_eq!(help_of("--version"), vec!["mr --version [--json]", "the version and the commit"]);
}
