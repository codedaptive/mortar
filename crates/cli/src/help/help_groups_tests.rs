//! HELP_GROUPS: four groups in the spec's order and headings, every verb of the table and --version once each, and
//! rows in the spec's words.
//! Unit test: the groups and the verb table are pure data.

use super::{HelpEntry, HELP_GROUPS};
use crate::args::VERBS;

fn entry(name: &str) -> &'static HelpEntry {
    HELP_GROUPS
        .iter()
        .flat_map(|group| group.entries.iter())
        .find(|entry| entry.name == name)
        .expect("the name is in the help")
}

#[test]
fn the_groups_are_agent_folder_landing_setup_with_the_spec_headings() {
    let groups: Vec<(&str, &str)> = HELP_GROUPS.iter().map(|group| (group.key, group.heading)).collect();
    assert_eq!(
        groups,
        vec![
            ("agent", "Agent verbs (run in your request folder)"),
            ("folder", "Folder and upkeep verbs (for the harness)"),
            ("landing", "Landing verbs (for the landing script)"),
            ("setup", "Setup"),
        ]
    );
}

#[test]
fn every_verb_and_version_is_listed_once() {
    let mut listed: Vec<&str> =
        HELP_GROUPS.iter().flat_map(|group| group.entries.iter()).map(|entry| entry.name).collect();
    listed.sort_unstable();
    let mut expected: Vec<&str> = VERBS.iter().map(|verb| verb.name).collect();
    expected.push("--version");
    expected.sort_unstable();
    assert_eq!(listed, expected);
}

#[test]
fn each_group_lists_its_verbs_in_the_spec_order() {
    let names: Vec<Vec<&str>> =
        HELP_GROUPS.iter().map(|group| group.entries.iter().map(|entry| entry.name).collect()).collect();
    assert_eq!(
        names,
        vec![
            vec!["status", "save", "join", "resolve", "need", "undo"],
            vec!["init", "folder", "snapshot", "overlap", "disk", "sweep", "log", "restore"],
            vec![
                "squash",
                "withdraw",
                "would-merge",
                "rereview",
                "land",
                "commit-files",
                "back-out",
                "publish",
                "after-publish",
                "guard",
                "guard-hook",
                "bisect",
                "backup",
                "restore-backup"
            ],
            vec!["install", "uninstall", "help", "--version"],
        ]
    );
}

#[test]
fn status_has_the_spec_line() {
    assert_eq!(
        entry("status"),
        &HelpEntry {
            name: "status",
            term: "status",
            line: "where you are, what you changed, whether your zone moved, anything conflicted, and the next step",
        }
    );
}

#[test]
fn folder_has_its_subcommands_as_term() {
    assert_eq!(
        entry("folder"),
        &HelpEntry {
            name: "folder",
            term: "folder zone|request|remove|attempts|keep|drop|refresh",
            line: "make, refresh and remove zone, request and attempt folders",
        }
    );
}

#[test]
fn guard_hook_is_run_by_git() {
    assert_eq!(
        entry("guard-hook"),
        &HelpEntry {
            name: "guard-hook",
            term: "guard-hook <name>",
            line: "run by git's hooks, not by hand; a refusal is exit 3, and git stops the push or the ref update it was preparing",
        }
    );
}

#[test]
fn version_has_the_spec_line_without_the_citation() {
    assert_eq!(
        entry("--version"),
        &HelpEntry { name: "--version", term: "--version", line: "the version and the commit" }
    );
}

#[test]
fn shared_rows_share_term_and_line() {
    for (first, second, term, line) in [
        (
            "install",
            "uninstall",
            "install, uninstall",
            "place mortar and mr and offer skill wiring; remove what install placed",
        ),
        ("backup", "restore-backup", "backup, restore-backup <host>", "make a backup; restore from one"),
    ] {
        for name in [first, second] {
            assert_eq!((entry(name).term, entry(name).line), (term, line), "{name}");
        }
    }
}
