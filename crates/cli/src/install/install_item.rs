//! One line of install's or uninstall's report (spec 07 item 8).
//! A bare type (S-7); item_line_tests and item_json_tests exercise it.

use std::path::PathBuf;

/// `action`: placed, replaced, unchanged, linked, link skipped, written, wiring skipped, not on PATH, removed, left,
/// gone or skipped. `path`: what it is about, empty when it is about no one path. `note`: why, or empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallItem {
    pub action: String,
    pub path: PathBuf,
    pub note: String,
}
