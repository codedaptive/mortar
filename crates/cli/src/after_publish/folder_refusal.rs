//! FolderRefusal: why a zone folder was not brought to its zone branch's tip, with nothing in it changed.
//! A bare type (S-7); zone_folder_lines_tests and after_publish_report_tests exercise each variant.

use std::path::PathBuf;

/// InTheWay: git's move refused over local changes at these paths. Unmerged: the folder's index has unmerged
/// entries at these paths. NoZoneCommit: the index's tree matches no value the zone branch has held (S-140).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FolderRefusal {
    InTheWay(Vec<PathBuf>),
    Unmerged(Vec<PathBuf>),
    NoZoneCommit,
}
