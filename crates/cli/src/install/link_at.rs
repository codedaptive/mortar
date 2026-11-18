//! What stands at `~/.local/bin/mr` before install links it (spec 07 item 3).
//! A bare type (S-7); plan_link_tests exercises it.

use std::path::PathBuf;

/// Absent: nothing there. Link: a symlink, with its target as read and whether it resolves to Mortar. Mortar: a
/// file that is not a link but is Mortar (S-147). Foreign: anything else, which install never replaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkAt {
    Absent,
    Link { target: PathBuf, mortar: bool },
    Mortar,
    Foreign,
}
