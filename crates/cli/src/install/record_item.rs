//! One item of install's record, `~/.local/state/mortar/install.json`.
//! A bare type; encode_record_tests, decode_record_tests and plan_uninstall_tests exercise it.

use std::path::PathBuf;

/// File: a file install wrote, with content_hash of what it wrote. Link: a symlink install made, with its target.
/// Dir: a folder install created (a folder that already existed is never recorded).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordItem {
    File { path: PathBuf, hash: String },
    Link { path: PathBuf, target: PathBuf },
    Dir { path: PathBuf },
}
