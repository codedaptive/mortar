// Arms whose text comes from gitio's or oplog's Display (Git, Oplog, ChangeIdMissing, Trailers, ChangeId, IndexNotSet,
// a Stale result holding an Oid) are not tested here: their text runs another crate's code, so they are cross-zone (rule 11).
use crate::SnapshotError;
use oplog::{OpPhase, OpRecord, OpResult};
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

fn record(phase: OpPhase, result: Option<OpResult>) -> OpRecord {
    OpRecord {
        op_id: "op-7".to_string(),
        key: "save refs/heads/x op-7".to_string(),
        kind: "save".to_string(),
        actor: "bilby".to_string(),
        reason: "save".to_string(),
        at: 1700000000,
        phase,
        moves: vec![],
        result,
    }
}

fn stale_absent() -> Option<OpResult> {
    Some(OpResult::Stale { refname: "refs/heads/x".to_string(), actual: None })
}

#[test]
fn unborn_head_names_the_folder() {
    let e = SnapshotError::UnbornHead { folder: PathBuf::from("/tmp/wt") };
    assert_eq!(e.to_string(), "HEAD of the folder /tmp/wt has no commit");
}

#[test]
fn detached_head_names_the_folder() {
    let e = SnapshotError::DetachedHead { folder: PathBuf::from("/tmp/wt") };
    assert_eq!(e.to_string(), "HEAD of the folder /tmp/wt names no branch");
}

#[test]
fn bad_target_gives_the_reason() {
    let e = SnapshotError::BadTarget { reason: "zone is empty".to_string() };
    assert_eq!(e.to_string(), "the snapshot ref cannot be named: zone is empty");
}

#[test]
fn bad_sentence_gives_the_reason() {
    let e = SnapshotError::BadSentence { reason: "it is blank".to_string() };
    assert_eq!(e.to_string(), "the save sentence is refused: it is blank");
}

#[test]
fn ref_moved_with_stale_absent_ref_says_absent() {
    let e = SnapshotError::RefMoved(record(OpPhase::Failed, stale_absent()));
    assert_eq!(e.to_string(), "the snapshot ref refs/heads/x was not at the value read (it holds absent); nothing moved");
}

#[test]
fn ref_moved_without_a_stale_result_names_the_op() {
    let e = SnapshotError::RefMoved(record(OpPhase::Failed, None));
    assert_eq!(e.to_string(), "the snapshot ref was not at the value read (op op-7); nothing moved");
}

#[test]
fn branch_moved_with_stale_absent_ref_says_absent() {
    let e = SnapshotError::BranchMoved(record(OpPhase::Failed, stale_absent()));
    assert_eq!(
        e.to_string(),
        "the branch refs/heads/x was not at the value read (it holds absent); the branch did not move"
    );
}

#[test]
fn branch_moved_with_a_moved_result_names_the_op() {
    let e = SnapshotError::BranchMoved(record(OpPhase::Failed, Some(OpResult::Moved)));
    assert_eq!(e.to_string(), "the branch was not at the value read (op op-7); the branch did not move");
}

#[test]
fn op_repeated_names_the_key_and_the_op() {
    let e = SnapshotError::OpRepeated(record(OpPhase::Done, Some(OpResult::Moved)));
    assert_eq!(
        e.to_string(),
        "the op log already holds a done record for key save refs/heads/x op-7 (op op-7); nothing moved"
    );
}

#[test]
fn file_size_names_the_path_and_shows_the_io_error() {
    let inner = || std::io::Error::other("too many links");
    let e = SnapshotError::FileSize { path: PathBuf::from("src/a.rs"), error: inner() };
    let expected = format!("the size of src/a.rs could not be read: {}; no commit was written", inner());
    assert_eq!(e.to_string(), expected);
}

#[test]
fn path_not_utf8_quotes_the_path() {
    let e = SnapshotError::PathNotUtf8 { path: PathBuf::from("src/a.rs") };
    assert_eq!(
        e.to_string(),
        "the refused file \"src/a.rs\" has a name that is not UTF-8, so it cannot be passed to git as an \
         exclude; no commit was written"
    );
}

#[test]
fn path_not_utf8_shows_a_non_utf8_byte_escaped() {
    let path = PathBuf::from(OsStr::from_bytes(b"bad\xffname"));
    assert_eq!(
        SnapshotError::PathNotUtf8 { path }.to_string(),
        "the refused file \"bad\\xFFname\" has a name that is not UTF-8, so it cannot be passed to git as an \
         exclude; no commit was written"
    );
}

#[test]
fn cap_mismatch_lists_the_paths_comma_separated() {
    let e = SnapshotError::CapMismatch { paths: vec![PathBuf::from("big.bin"), PathBuf::from("dir/other.txt")] };
    assert_eq!(
        e.to_string(),
        "the capped tree did not differ from the full tree by exactly the refused files: big.bin, dir/other.txt; \
         no commit was written"
    );
}
