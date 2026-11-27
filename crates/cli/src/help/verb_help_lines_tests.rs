//! verb_help_lines: a verb's usage, then its row's line, then its notes (squash's and publish's are checked by token
//! in verb_notes_tests; folder's, would-merge's and commit-files's are pinned here in full), then its tokens saved
//! baseline if it has one. would-merge has both a note and a baseline, so its help pins their order: the note, then
//! the baseline.
//! Unit test: the help groups, find_help_entry, usage_forms, verb_notes and the verb table are pure and used directly.

use super::{find_help_entry, verb_help_lines, verb_notes};

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
fn rereview_and_overlap_name_their_baseline() {
    for (verb, words) in [
        ("rereview", "git diff <reviewed> <new>"),
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
fn folder_is_its_forms_its_line_then_its_note() {
    assert_eq!(
        help_of("folder"),
        vec![
            "mr folder zone <zone> <path> [--start <commit>] --actor <name> [--key <operation key>] [--json]",
            "mr folder request <zone> <request> <path> --paths <path>... [--with <zone>]... --actor <name> [--key \
             <operation key>] [--json]",
            "mr folder remove <path> --actor <name> [--key <operation key>] [--json]",
            "mr folder attempts <n> <dir> --actor <name> [--key <operation key>] [--json]",
            "mr folder keep <k> --actor <name> [--key <operation key>] [--json]",
            "mr folder drop --actor <name> [--key <operation key>] [--json]",
            "mr folder refresh <zone folder path> --actor <name> [--key <operation key>] [--json]",
            "make, refresh and remove zone, request and attempt folders",
            "refresh <zone folder path>: sets the zone folder's paths to the zone's paths and root files in the \
             config and brings the folder to the zone branch's tip, so a zone's tests can run between publishes",
            "request --with <zone>: cuts the request on a merge of its zone's tip with each partner zone's tip, \
             in the order given; a merge that conflicts is refused, naming the partner and the files; mr join then \
             gives the zone only the request's own commits",
        ]
    );
}

#[test]
fn would_merge_is_its_usage_its_line_its_note_then_its_baseline() {
    assert_eq!(
        help_of("would-merge"),
        vec![
            "mr would-merge <commit or change ID> [--with <commit or change ID>]... --actor <name> [--key <operation \
             key>] [--json]",
            "say whether it would merge into integration",
            "--with <commit or change ID>: each further squash, in the order given, merges onto the result of the \
             one before, as successive lands would; a conflict names its step (1 is the first squash) and its files; \
             no ref moves",
            "tokens saved is counted against: git log -p <base>..<commit>",
        ]
    );
}

#[test]
fn squash_is_its_usage_its_line_then_its_notes_and_no_baseline() {
    let lines = help_of("squash");
    assert_eq!(
        lines[..2],
        [
            "mr squash <zone> [--message <text>] --actor <name> [--key <operation key>] [--json]",
            "make a zone's squash",
        ]
    );
    assert_eq!(lines[2..], *verb_notes("squash"));
}

#[test]
fn commit_files_is_its_usage_its_line_then_its_two_notes() {
    assert_eq!(
        help_of("commit-files"),
        vec![
            "mr commit-files <path>... [--message <text>] --actor <name> [--key <operation key>] [--json]",
            "commit named files onto the zone, or onto integration inside a landing",
            "in a zone folder at its zone's tip: one commit of the named paths on zone/<zone>, under join-<zone>, \
             refused while the folder's index holds staged changes, except a stopped run's own paths, which answer \
             same with next git restore --staged (unstaged edits are left alone); in the integration folder after \
             mr land: one commit on integration inside the landing, under land; refused in a request or attempt \
             folder (mr save)",
            "--message <text>: the commit's message; without it, the message names the paths; the folder's other \
             changes are left as they are",
        ]
    );
}

#[test]
fn withdraw_is_its_usage_and_its_line() {
    assert_eq!(
        help_of("withdraw"),
        vec![
            "mr withdraw <zone> --actor <name> [--key <operation key>] [--json]",
            "withdraw a zone's squash before it lands",
        ]
    );
}

#[test]
fn version_is_its_usage_and_its_line() {
    assert_eq!(help_of("--version"), vec!["mr --version [--json]", "the version and the commit"]);
}
