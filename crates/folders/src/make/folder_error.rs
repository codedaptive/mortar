//! Every way making a folder, making attempt folders with make_attempts, keeping one with keep, dropping them with
//! drop_attempts, remaking orphan attempt folders with remake_orphans, widening one with need, moving one with
//! move_folder, removing one with remove, or measuring or sweeping with disk, sweep_build or sweep_refs can be
//! refused.
//! A bare type; folder_error_display_tests exercises each variant's text, folder_error_source_tests its source.

use std::path::PathBuf;

use gitio::{GitError, Oid};
use oplog::{LockError, OplogError};

use crate::plan::{BelowFloor, PlanError};
use crate::record::{FolderRecord, RecordError};

/// A refusal from make_zone_folder, make_request_folder, make_attempts, keep, drop_attempts, remake_orphans, need,
/// move_folder, remove, disk, sweep_build or sweep_refs. Debug only: io::Error, OplogError and LockError are neither
/// Clone nor Eq.
#[derive(Debug)]
pub enum FolderError {
    /// A zone or request name, a sparse path, or an attempt count of 0, the plan refused.
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
    /// The request branch (short name) that attempts are cut from, or that keep moves, does not exist.
    RequestBranchMissing { branch: String },
    /// keep found no attempt branch (short name) for the attempt asked for.
    AttemptMissing { branch: String },
    /// keep or drop_attempts refused: the attempts at `paths`, each named by its folder, or by its full branch name
    /// when it has no folder, are not marked saved by the caller (S-32); nothing was touched.
    AttemptsUnsaved { paths: Vec<PathBuf> },
    /// remove refused the request folder at `path`: attempt branches (full names) or attempt folders (paths, as
    /// display text) of its request remain (S-82); nothing changed.
    AttemptsRemain { path: PathBuf, attempts: Vec<String> },
    /// keep or drop_attempts refused: the attempt folders at `paths` have lost their try branch (orphans); nothing
    /// was touched; next step: remake_orphans, which the cli runs (S-96).
    OrphanAttempt { paths: Vec<PathBuf> },
    /// remake_orphans refused: the orphan attempt folders at `paths` have no trysnap ref, or its commit has no
    /// parent, so Mortar holds no copy of them; nothing changed; next step: copy out anything wanted, then remove
    /// each folder by hand.
    OrphanWithoutSnapshot { paths: Vec<PathBuf> },
    /// keep refused: request branch `branch` has commits the kept attempt's branch `attempt` does not hold, so
    /// moving it would drop them; nothing was touched.
    RequestMovedOn { branch: String, attempt: String },
    /// remove refused the folder at `path`: it is attempt `k`'s folder, and keep removes attempt folders.
    AttemptFolder { path: PathBuf, k: u32 },
    /// The folder at `path` holds work that is not saved; nothing ran.
    NotSaved { path: PathBuf },
    /// git refused to move the folder at `path` over local changes at `paths`; nothing changed, nothing logged.
    MoveRefused { path: PathBuf, paths: Vec<PathBuf> },
    /// The folder's branch is at neither tip of the move; `at` is None when it has no commit.
    BranchElsewhere {
        branch: String,
        at: Option<Oid>,
        old: Oid,
        new: Oid,
    },
    /// remove refused the folder at `path`: its work is not saved, or the snapshot size cap (`cap` bytes, None
    /// when the caller does not know it) refused the files at `refused`, so they are in no snapshot (S-32).
    NotRemovedUnsaved {
        path: PathBuf,
        refused: Vec<PathBuf>,
        cap: Option<u64>,
    },
    /// remove refused the folder at `path`: its branch is not contained in its zone branch, so its work has not
    /// joined. Both are full ref names.
    NotJoined {
        path: PathBuf,
        branch: String,
        zone_branch: String,
    },
    /// remove refused the folder at `path`: it is `zone`'s standing folder, and remove takes request folders only.
    ZoneFolder { path: PathBuf, zone: String },
    /// The ref `refname` was not where it was read when sweep_refs' delete op ran, so the op log deleted nothing;
    /// `at` is where it is now (None when absent). keep and drop_attempts return AttemptRefsLeft for this case, and
    /// remove (delete_folder_refs) returns RemovedRefsLeft.
    RefMoved { refname: String, at: Option<Oid> },
    /// keep or drop_attempts removed the attempt folders at `removed` (empty when none was left to remove), then its
    /// op moved nothing because the ref `moved` was not where it was read (`at`: where it is now, None when absent);
    /// every attempt branch is kept.
    AttemptRefsLeft {
        moved: String,
        at: Option<Oid>,
        removed: Vec<PathBuf>,
    },
    /// remove took the folder at `path` away, but the ref `moved` was not where remove read it when the delete op
    /// ran (`at`: where it is now, None when absent), so neither `branch` nor its snapshot ref was deleted. The
    /// branch is already contained in its zone, so the work they hold is in the zone; mortar sweep (sweep_refs)
    /// deletes both once nothing holds the branch (S-52 (1)).
    RemovedRefsLeft {
        path: PathBuf,
        branch: String,
        moved: String,
        at: Option<Oid>,
    },
    /// sweep_refs refused the caller's extra ref `refname` for `reason` (a phrase that follows the name): it is
    /// under refs/heads/, refs/mortar/ or refs/remotes/, or protected (S-53, S-54); nothing was swept.
    ExtraRefRefused { refname: String, reason: String },
    /// A file system call on `path` failed.
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
}
