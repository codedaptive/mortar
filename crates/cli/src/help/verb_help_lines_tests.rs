//! verb_help_lines: a verb's usage one line per form, then its row's line, then its tokens saved baseline if it has
//! one.
//! Unit test: the help groups, find_help_entry, usage_forms and the verb table are pure and used directly.

use super::{find_help_entry, verb_help_lines};

fn help_of(name: &str) -> Vec<String> {
    let (_, entry) = find_help_entry(name).expect("the verb is in the help");
    verb_help_lines(entry)
}

#[test]
fn status_is_its_usage_its_line_and_its_baseline() {
    assert_eq!(
        help_of("status"),
        vec![
            "mr status [--json]",
            "where you are, what you changed, whether your zone moved, anything conflicted, and the next step",
            "tokens saved is counted against: git status and git diff HEAD over this folder",
        ]
    );
}

#[test]
fn rereview_would_merge_and_overlap_name_their_baseline() {
    for (verb, words) in [
        ("rereview", "git diff <reviewed> <new>"),
        ("would-merge", "git log -p <base>..<commit>"),
        ("overlap", "git log -p of each request's range"),
    ] {
        let expected = format!("tokens saved is counted against: {words}");
        assert_eq!(help_of(verb).last(), Some(&expected), "{verb}");
    }
}

#[test]
fn log_has_no_baseline_line() {
    assert_eq!(
        help_of("log"),
        vec!["mr log [--ref <full ref name>] [--actor <name>] [--key <operation key>] [--json]", "the operation log"]
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
