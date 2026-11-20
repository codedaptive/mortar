//! What uninstall will remove, decided from the record and what is on disk.
//! A bare type; plan_uninstall_tests exercises it.

use std::path::PathBuf;

use super::InstallItem;

/// `remove`: the files and links still as install placed them. `dirs`: the recorded folders on install's map that are
/// still folders, deepest first, removed when empty: those holding the record after it, the rest before it. `items`:
/// the report's lines for the record's items, except the folders in `dirs` (their lines come when they are tried)
/// and gone folders (no line).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UninstallPlan {
    pub remove: Vec<PathBuf>,
    pub dirs: Vec<PathBuf>,
    pub items: Vec<InstallItem>,
}
