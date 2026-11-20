//! setup_group: the program help's setup verbs and `--version`, in the spec's words.
//! A bare const; help_groups_tests covers it.

use super::{HelpEntry, HelpGroup};

/// install and uninstall share one row of the spec's table.
const INSTALL_TERM: &str = "install, uninstall";
const INSTALL_LINE: &str = "place mortar and mr and offer skill wiring; remove what install placed";

pub const SETUP_GROUP: HelpGroup = HelpGroup {
    key: "setup",
    heading: "Setup",
    entries: &[
        HelpEntry { name: "install", term: INSTALL_TERM, line: INSTALL_LINE },
        HelpEntry { name: "uninstall", term: INSTALL_TERM, line: INSTALL_LINE },
        HelpEntry { name: "help", term: "help [<verb>]", line: "this list, or one verb's usage" },
        HelpEntry { name: "--version", term: "--version", line: "the version and the commit" },
    ],
};
