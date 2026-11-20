//! What stopped an integration folder move, as folder_move_refusal_report shows it.
//! A bare type: folder_move_refusal_report_tests exercises both variants.

use std::path::PathBuf;

/// The paths git found local changes at (folders' FolderError::MoveRefused), which the refusal lists one per line,
/// or any other reason, in the cli's words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveBlock {
    Paths(Vec<PathBuf>),
    Other(String),
}
