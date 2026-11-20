//! FolderHolds: what a folder holds that `mr disk` reports beside its bytes.
//! A bare type; holds_line_tests, holds_json_tests and disk_report_tests build it.

use folders::UnsavedCounts;

use super::JoinedAnswer;

/// The folder's `unsaved` counts (folders::folder_unsaved) and its `joined` answer: joined or not (status's rule),
/// none for a zone folder, or unknown with why when the joined read failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderHolds {
    pub unsaved: UnsavedCounts,
    pub joined: JoinedAnswer,
}
