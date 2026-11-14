//! SnapshotError: why a snapshot or a save did not happen. A declaration only; snapshot_ref_name_tests exercises
//! BadTarget, snapshot_from_apply_tests OpRepeated and RefMoved, capped_excludes_tests PathNotUtf8, save_head_tests
//! UnbornHead and DetachedHead, save_message_tests BadSentence, save_change_id_tests ChangeIdMissing, and
//! save_from_apply_tests OpRepeated and BranchMoved. Nagatha's integration tests get, through snapshot: RefMoved in
//! tests/snapshot/race.rs::foreign_logged_move_after_previous_read_is_refused_without_overwriting_ref_or_files;
//! FileSize (an ELOOP) in
//! tests/snapshot/sizing_edges.rs::real_metadata_symlink_loop_is_file_size_error_before_commit_ref_or_log_and_retry_preserves_work;
//! CapMismatch, also through is_saved, in
//! sizing_edges.rs::caller_noglob_environment_produces_exact_cap_mismatch_without_losing_previous_snapshot;
//! UnbornHead, also through is_saved, in
//! sizing_edges.rs::unborn_head_is_named_before_any_snapshot_commit_ref_or_op.
//! ChangeId and IndexNotSet come only from I/O, and Trailers from I/O or a malformed Mortar-Change-Id in HEAD's
//! message; none is yet covered by an integration test.

use gitio::Oid;
use std::path::PathBuf;

/// Debug only: oplog's OplogError is neither Clone nor Eq.
#[derive(Debug)]
pub enum SnapshotError {
    /// A git run failed.
    Git(gitio::GitError),
    /// The op log refused.
    Oplog(oplog::OplogError),
    /// The folder's HEAD has no commit yet, so there is nothing to start the tree from or to save onto.
    UnbornHead { folder: PathBuf },
    /// The folder's HEAD names no branch, so save has no branch to move.
    DetachedHead { folder: PathBuf },
    /// A zone or request that cannot form the ref name; the reason names it.
    BadTarget { reason: String },
    /// The sentence cannot be a commit subject; the reason says why.
    BadSentence { reason: String },
    /// HEAD's Mortar-Request and Mortar-Zone are the target's but it carries no Mortar-Change-Id (S-34); save writes
    /// no commit and moves no branch rather than make a second ID for the request. Its snapshot step has already
    /// run, so the request's snapshot ref may have moved.
    ChangeIdMissing { head: Oid },
    /// HEAD's message trailers could not be read.
    Trailers(oplog::MortarTrailerError),
    /// A new change ID could not be drawn.
    ChangeId(oplog::ChangeIdError),
    /// The snapshot ref was not at the value read before the move, so nothing moved; the failed record names the
    /// ref and its actual value.
    RefMoved(oplog::OpRecord),
    /// Save's branch was not at the HEAD commit read, so nothing moved; the failed record names the ref and its
    /// actual value.
    BranchMoved(oplog::OpRecord),
    /// The op log already held a done record for this op's key, so nothing moved and the ref was not checked; the
    /// record is that earlier done record. With a key made from a fresh op id this means an op id was reused.
    OpRepeated(oplog::OpRecord),
    /// The branch moved to `commit`, then setting the folder's index failed; the files are untouched and the index
    /// is the old one. A save again finds the tree equal to HEAD's, writes no commit and leaves the index as it is.
    IndexNotSet { commit: Oid, error: gitio::GitError },
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
