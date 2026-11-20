//! find_help_entry: a row of the program help by its verb name, with the group it is in.

use super::{HelpEntry, HelpGroup, HELP_GROUPS};

/// The group and row whose name is exactly `name` (a verb table name or "--version"); None for any other word.
pub fn find_help_entry(name: &str) -> Option<(&'static HelpGroup, &'static HelpEntry)> {
    HELP_GROUPS
        .iter()
        .find_map(|group| group.entries.iter().find(|entry| entry.name == name).map(|entry| (group, entry)))
}
