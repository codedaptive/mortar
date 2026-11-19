use crate::saved_with_snapshot_from::saved_with_snapshot_from;
use crate::{RefusedFile, SaveOutcome, SavedWithSnapshot, SnapshotOutcome};
use gitio::Oid;
use oplog::ChangeId;
use std::path::PathBuf;

const HEAD: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const WRITTEN: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const SNAPSHOT_COMMIT: &str = "cccccccccccccccccccccccccccccccccccccccc";
const TREE: &str = "dddddddddddddddddddddddddddddddddddddddd";
const CHANGE_ID: &str = "kkkkllllmmmmnnnnooooppppqqqqrrrr";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn big_file() -> RefusedFile {
    RefusedFile { path: PathBuf::from("assets/big.bin"), size: 9_000_000, cap: 5_242_880 }
}

fn snap(written: bool) -> SnapshotOutcome {
    SnapshotOutcome {
        commit: oid(SNAPSHOT_COMMIT),
        tree: oid(TREE),
        written,
        refused: vec![big_file()],
        formatted: None,
    }
}

#[test]
fn unchanged_save_after_a_written_snapshot_says_the_snapshot_ref_moved() {
    assert_eq!(
        saved_with_snapshot_from(snap(true), oid(HEAD), false, None),
        SavedWithSnapshot {
            outcome: SaveOutcome {
                commit: oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
                tree: oid("dddddddddddddddddddddddddddddddddddddddd"),
                written: false,
                change_id: None,
                refused: vec![RefusedFile {
                    path: PathBuf::from("assets/big.bin"),
                    size: 9_000_000,
                    cap: 5_242_880
                }],
            },
            snapshot_written: true,
        }
    );
}

#[test]
fn unchanged_save_after_an_unchanged_snapshot_says_the_snapshot_ref_stayed() {
    assert_eq!(
        saved_with_snapshot_from(snap(false), oid(HEAD), false, None),
        SavedWithSnapshot {
            outcome: SaveOutcome {
                commit: oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
                tree: oid("dddddddddddddddddddddddddddddddddddddddd"),
                written: false,
                change_id: None,
                refused: vec![RefusedFile {
                    path: PathBuf::from("assets/big.bin"),
                    size: 9_000_000,
                    cap: 5_242_880
                }],
            },
            snapshot_written: false,
        }
    );
}

#[test]
fn written_save_after_a_written_snapshot_carries_its_commit_and_change_id() {
    assert_eq!(
        saved_with_snapshot_from(snap(true), oid(WRITTEN), true, Some(ChangeId::parse(CHANGE_ID).unwrap())),
        SavedWithSnapshot {
            outcome: SaveOutcome {
                commit: oid("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
                tree: oid("dddddddddddddddddddddddddddddddddddddddd"),
                written: true,
                change_id: Some(ChangeId::parse("kkkkllllmmmmnnnnooooppppqqqqrrrr").unwrap()),
                refused: vec![RefusedFile {
                    path: PathBuf::from("assets/big.bin"),
                    size: 9_000_000,
                    cap: 5_242_880
                }],
            },
            snapshot_written: true,
        }
    );
}

#[test]
fn written_save_after_an_unchanged_snapshot_says_the_snapshot_ref_stayed() {
    assert_eq!(
        saved_with_snapshot_from(snap(false), oid(WRITTEN), true, Some(ChangeId::parse(CHANGE_ID).unwrap())),
        SavedWithSnapshot {
            outcome: SaveOutcome {
                commit: oid("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
                tree: oid("dddddddddddddddddddddddddddddddddddddddd"),
                written: true,
                change_id: Some(ChangeId::parse("kkkkllllmmmmnnnnooooppppqqqqrrrr").unwrap()),
                refused: vec![RefusedFile {
                    path: PathBuf::from("assets/big.bin"),
                    size: 9_000_000,
                    cap: 5_242_880
                }],
            },
            snapshot_written: false,
        }
    );
}

#[test]
fn an_empty_refused_list_passes_through_empty() {
    let snap = SnapshotOutcome { refused: vec![], ..snap(true) };
    assert_eq!(
        saved_with_snapshot_from(snap, oid(HEAD), false, None),
        SavedWithSnapshot {
            outcome: SaveOutcome {
                commit: oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
                tree: oid("dddddddddddddddddddddddddddddddddddddddd"),
                written: false,
                change_id: None,
                refused: vec![],
            },
            snapshot_written: true,
        }
    );
}
