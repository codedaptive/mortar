//! agent_group: the program help's agent verbs, in the spec's words (SPEC-cli 08, S-175).
//! A bare const (S-7); help_groups_tests covers it.

use super::{HelpEntry, HelpGroup};

pub const AGENT_GROUP: HelpGroup = HelpGroup {
    key: "agent",
    heading: "Agent verbs (run in your request folder)",
    entries: &[
        HelpEntry {
            name: "status",
            term: "status",
            line: "where you are, what you changed, whether your zone moved, anything conflicted, and the next step",
        },
        HelpEntry { name: "save", term: "save \"<sentence>\"", line: "keep your work as a commit" },
        HelpEntry { name: "join", term: "join", line: "join your request to its zone; rebased if the zone moved" },
        HelpEntry {
            name: "resolve",
            term: "resolve",
            line: "write the conflicted files into your folder with markers; run again when fixed",
        },
        HelpEntry { name: "need", term: "need <path>", line: "add a path to your folder" },
        HelpEntry { name: "undo", term: "undo", line: "take back your own last step" },
    ],
};
