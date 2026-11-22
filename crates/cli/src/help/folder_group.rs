//! folder_group: the program help's folder and upkeep verbs, in the spec's words.
//! A bare const; help_groups_tests covers it.

use super::{HelpEntry, HelpGroup};

pub const FOLDER_GROUP: HelpGroup = HelpGroup {
    key: "folder",
    heading: "Folder and upkeep verbs (for the harness)",
    entries: &[
        HelpEntry { name: "init", term: "init", line: "write this repository's Mortar config" },
        HelpEntry {
            name: "folder",
            term: "folder zone|request|remove|attempts|keep|drop|refresh",
            line: "make, refresh and remove zone, request and attempt folders",
        },
        HelpEntry { name: "snapshot", term: "snapshot", line: "save the folder's state to its hidden ref" },
        HelpEntry {
            name: "overlap",
            term: "overlap <zone>",
            line: "which open requests in a zone change the same files",
        },
        HelpEntry { name: "disk", term: "disk", line: "disk use per folder, the store, and what a sweep frees" },
        HelpEntry {
            name: "sweep",
            term: "sweep",
            line: "clear refs a sweep may drop, and idle build output with --idle-days",
        },
        HelpEntry { name: "log", term: "log", line: "the operation log" },
        HelpEntry { name: "restore", term: "restore <op id> <ref>...", line: "restore refs from the operation log" },
    ],
};
