//! help_entry: one row of the program help.
//! A bare type; help_groups_tests and find_help_entry_tests exercise it.

/// `name`: the verb table's name, or "--version". `term`: the row's first column as printed, shared by the verbs
/// one row covers (install and uninstall; backup and restore-backup). `line`: the row's words.
#[derive(Debug, PartialEq, Eq)]
pub struct HelpEntry {
    pub name: &'static str,
    pub term: &'static str,
    pub line: &'static str,
}
