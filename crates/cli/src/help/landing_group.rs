//! landing_group: the program help's landing verbs, in the spec's words.
//! A bare const; help_groups_tests covers it.

use super::{HelpEntry, HelpGroup};

/// backup and restore-backup share one row of the spec's table.
const BACKUP_TERM: &str = "backup, restore-backup <host>";
const BACKUP_LINE: &str = "make a backup; restore from one";

pub const LANDING_GROUP: HelpGroup = HelpGroup {
    key: "landing",
    heading: "Landing verbs (for the landing script)",
    entries: &[
        HelpEntry { name: "squash", term: "squash <zone> [--message <text>]", line: "make a zone's squash" },
        HelpEntry { name: "withdraw", term: "withdraw <zone>", line: "withdraw a zone's squash before it lands" },
        HelpEntry {
            name: "would-merge",
            term: "would-merge <commit or change ID>",
            line: "say whether it would merge into integration",
        },
        HelpEntry {
            name: "rereview",
            term: "rereview <reviewed commit> <new commit>",
            line: "the diff since a reviewed squash",
        },
        HelpEntry { name: "land", term: "land <commit>", line: "land a squash on integration" },
        HelpEntry { name: "back-out", term: "back-out", line: "put integration back on origin" },
        HelpEntry {
            name: "publish",
            term: "publish [--no-push]",
            line: "push integration to origin; with --no-push, move the local origin only",
        },
        HelpEntry {
            name: "after-publish",
            term: "after-publish <zone> [--move <zone>]...",
            line: "bring zones onto the new origin",
        },
        HelpEntry {
            name: "guard",
            term: "guard install|status",
            line: "the git hooks that keep integration and origin to Mortar",
        },
        HelpEntry { name: "guard-hook", term: "guard-hook <name>", line: "run by git's hooks, not by hand" },
        HelpEntry {
            name: "bisect",
            term: "bisect <last pass> <failed> --timeout <s> -- <command>...",
            line: "find the landing that broke a test",
        },
        HelpEntry { name: "backup", term: BACKUP_TERM, line: BACKUP_LINE },
        HelpEntry { name: "restore-backup", term: BACKUP_TERM, line: BACKUP_LINE },
    ],
};
