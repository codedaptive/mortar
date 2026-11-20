//! FolderHolds: what a folder holds that `mr disk` reports beside its bytes.
//! A bare type; holds_line_tests, holds_json_tests and disk_report_tests build it.

use folders::UnsavedCounts;

/// The folder's `unsaved` counts (folders::folder_unsaved) and whether its request has joined its zone (`joined`,
/// status's rule); `joined` is None for a zone folder, which has no request and so no joined answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FolderHolds {
    pub unsaved: UnsavedCounts,
    pub joined: Option<bool>,
}
