//! Every way making a folder, or widening one with need, can be refused.
//! A bare type; folder_error_display_tests exercises each variant's text, folder_error_source_tests its source.

use std::path::PathBuf;

use gitio::GitError;
use oplog::{LockError, OplogError};

use crate::plan::{BelowFloor, PlanError};
use crate::record::{FolderRecord, RecordError};

/// A refusal from make_zone_folder, make_request_folder or need. Debug only: io::Error, OplogError and LockError are
/// neither Clone nor Eq.
#[derive(Debug)]
pub enum FolderError {
    /// A zone or request name, or a sparse path, the plan refused.
    Plan(PlanError),
    /// The folder record at `path` is not one mortar writes.
    Record { path: PathBuf, error: RecordError },
    /// A git run failed.
    Git(GitError),
    /// The op log refused to log or apply an op.
    Oplog(OplogError),
    /// The folders lock could not be taken or released, or this host's name could not be read.
    Lock(LockError),
    /// Free disk is under the floor; nothing was written.
    BelowFloor(BelowFloor),
    /// Free disk at `path` could not be read.
    DiskProbe { path: PathBuf, reason: String },
    /// `path` is already a folder, and its record differs from the one asked for in more than its base and needed
    /// paths.
    DifferentFolder { path: PathBuf, existing: FolderRecord },
    /// `path` is in the way and is not a folder mortar made.
    NotAFolder { path: PathBuf, reason: String },
    /// The folder's branch is checked out in the worktree at `path`.
    BranchCheckedOut { branch: String, path: PathBuf },
    /// The folder's branch already exists at another commit, or was not made by mortar at this one, so it
    /// could not be made.
    BranchExists { branch: String },
    /// The zone branch a folder is cut from does not exist.
    ZoneBranchMissing { branch: String },
    /// A file system call on `path` failed.
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
}
