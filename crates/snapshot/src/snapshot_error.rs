//! SnapshotError: why a snapshot did not happen. A declaration only; snapshot_ref_name_tests exercises BadTarget,
//! snapshot_from_apply_tests OpRepeated and RefMoved, capped_excludes_tests PathNotUtf8. Nagatha's integration test
//! tests/snapshot/race.rs::foreign_logged_move_after_previous_read_is_refused_without_overwriting_ref_or_files gets
//! RefMoved through snapshot. FileSize and CapMismatch come only from I/O, and are not yet covered by an
//! integration test.

use std::path::PathBuf;

/// Debug only: oplog's OplogError is neither Clone nor Eq.
#[derive(Debug)]
pub enum SnapshotError {
    /// A git run failed.
    Git(gitio::GitError),
    /// The op log refused.
    Oplog(oplog::OplogError),
    /// The folder's HEAD has no commit yet, so there is nothing to start the tree from.
    UnbornHead { folder: PathBuf },
    /// A zone or request that cannot form the ref name; the reason names it.
    BadTarget { reason: String },
    /// The snapshot ref was not at the value read before the move, so nothing moved; the failed record names the
    /// ref and its actual value.
    RefMoved(oplog::OpRecord),
    /// The op log already held a done record for this op's key, so nothing moved and the ref was not checked; the
    /// record is that earlier done record. With a key made from a fresh op id this means an op id was reused.
    OpRepeated(oplog::OpRecord),
    /// A changed file's size could not be read, so the cap could not be checked; no commit was written.
    FileSize { path: PathBuf, error: std::io::Error },
    /// A refused path is not UTF-8, so it cannot be passed to git as an exclude; no commit was written.
    PathNotUtf8 { path: PathBuf },
    /// The capped tree did not differ from the full tree by exactly the refused files: git matched an exclude
    /// differently (e.g. GIT_NOGLOB_PATHSPECS or GIT_LITERAL_PATHSPECS set), or the folder changed between the two
    /// tree builds; no commit was written. The paths are
    /// the refused files still in the tree, then the other files left out of it.
    CapMismatch { paths: Vec<PathBuf> },
}
