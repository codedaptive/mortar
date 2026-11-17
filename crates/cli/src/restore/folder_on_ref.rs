//! A Mortar folder standing on a ref an `mr restore` names.
//! A bare type (S-7); restore_refusal_tests and restore_report_tests exercise it.

use std::path::PathBuf;

use folders::FolderRecord;

/// `path` is the worktree's folder as git lists it, `refname` its branch (a full name, one of the refs named) and
/// `record` its folder record: a zone folder's kind is Zone (S-115(1)).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderOnRef {
    pub path: PathBuf,
    pub refname: String,
    pub record: FolderRecord,
}
