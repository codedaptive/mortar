//! ZoneFolderOutcome: what became of one named zone's folder after land answered (S-139, S-140).
//! A bare type (S-7); zone_folder_lines_tests and after_publish_report_tests exercise each variant.

use std::path::PathBuf;

use super::FolderRefusal;

/// NoFolder: the zone has no zone folder. AtTip: its index already held the tip's tree; nothing moved. Moved: its
/// files and index were brought to the tip. Refused: nothing in it changed, for `why`. Failed: git failed on it;
/// `why` says what, in the cli's words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZoneFolderOutcome {
    NoFolder,
    AtTip { folder: PathBuf },
    Moved { folder: PathBuf },
    Refused { folder: PathBuf, why: FolderRefusal },
    Failed { folder: PathBuf, why: String },
}
