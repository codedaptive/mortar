//! help_group: one group of the program help, its heading and its rows.
//! A bare type; help_groups_tests and find_help_entry_tests exercise it.

use super::HelpEntry;

/// `key`: the group's name in the `--json` object ("agent", "folder", "landing", "setup"). `heading`: the line
/// printed above its rows. `entries`: its rows in the spec's order.
#[derive(Debug, PartialEq, Eq)]
pub struct HelpGroup {
    pub key: &'static str,
    pub heading: &'static str,
    pub entries: &'static [HelpEntry],
}
