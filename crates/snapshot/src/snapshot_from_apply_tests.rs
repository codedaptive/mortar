use crate::snapshot_from_apply::snapshot_from_apply;
use crate::{FormatterResult, FormatterRun, RefusedFile, SnapshotError, SnapshotOutcome};
use std::ffi::OsString;
use std::path::PathBuf;
use gitio::Oid;
use oplog::{ApplyOutcome, OpPhase, OpRecord, OpResult, RefMove};

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn record(phase: OpPhase, result: OpResult) -> OpRecord {
    OpRecord {
        op_id: "op-1".to_string(),
        key: "snapshot refs/mortar/snap/snapshot/WO-snapshot-1 op-1".to_string(),
        kind: "snapshot".to_string(),
        actor: "bilby".to_string(),
        reason: "snapshot".to_string(),
        at: 1700000000,
        phase,
        moves: vec![RefMove {
            refname: "refs/mortar/snap/snapshot/WO-snapshot-1".to_string(),
            old: None,
            new: oid("dddddddddddddddddddddddddddddddddddddddd"),
        }],
        result: Some(result),
    }
}

#[test]
fn done_is_the_commit_and_tree_written_with_the_refused_files() {
    let outcome = snapshot_from_apply(
        ApplyOutcome::Done(record(OpPhase::Done, OpResult::Moved)),
        oid("dddddddddddddddddddddddddddddddddddddddd"),
        oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        vec![RefusedFile { path: PathBuf::from("assets/big.bin"), size: 6_291_456, cap: 5_242_880 }],
        None,
    );
    assert_eq!(
        outcome.unwrap(),
        SnapshotOutcome {
            commit: oid("dddddddddddddddddddddddddddddddddddddddd"),
            tree: oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            written: true,
            refused: vec![RefusedFile { path: PathBuf::from("assets/big.bin"), size: 6_291_456, cap: 5_242_880 }],
            formatted: None,
        }
    );
}

#[test]
fn done_carries_a_failed_formatter_run_and_is_still_written() {
    let outcome = snapshot_from_apply(
        ApplyOutcome::Done(record(OpPhase::Done, OpResult::Moved)),
        oid("dddddddddddddddddddddddddddddddddddddddd"),
        oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        Vec::new(),
        Some(FormatterRun {
            program: PathBuf::from("rustfmt"),
            args: vec![OsString::from("--")],
            paths: vec![PathBuf::from("src/a.rs")],
            result: FormatterResult::Failed { code: Some(1), stderr: "error: src/a.rs\n".to_string() },
        }),
    );
    assert_eq!(
        outcome.unwrap(),
        SnapshotOutcome {
            commit: oid("dddddddddddddddddddddddddddddddddddddddd"),
            tree: oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            written: true,
            refused: Vec::new(),
            formatted: Some(FormatterRun {
                program: PathBuf::from("rustfmt"),
                args: vec![OsString::from("--")],
                paths: vec![PathBuf::from("src/a.rs")],
                result: FormatterResult::Failed { code: Some(1), stderr: "error: src/a.rs\n".to_string() },
            }),
        }
    );
}

#[test]
fn repeated_is_op_repeated_with_the_earlier_done_record() {
    let done = record(OpPhase::Done, OpResult::Moved);
    let outcome = snapshot_from_apply(
        ApplyOutcome::Repeated(done.clone()),
        oid("dddddddddddddddddddddddddddddddddddddddd"),
        oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        Vec::new(),
        None,
    );
    match outcome {
        Err(SnapshotError::OpRepeated(carried)) => assert_eq!(carried, done),
        other => panic!("expected OpRepeated, got {other:?}"),
    }
}

#[test]
fn failed_is_ref_moved_with_the_failed_record() {
    let failed = record(
        OpPhase::Failed,
        OpResult::Stale {
            refname: "refs/mortar/snap/snapshot/WO-snapshot-1".to_string(),
            actual: Some(oid("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee")),
        },
    );
    let outcome = snapshot_from_apply(
        ApplyOutcome::Failed(failed.clone()),
        oid("dddddddddddddddddddddddddddddddddddddddd"),
        oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        Vec::new(),
        None,
    );
    match outcome {
        Err(SnapshotError::RefMoved(carried)) => assert_eq!(carried, failed),
        other => panic!("expected RefMoved, got {other:?}"),
    }
}
