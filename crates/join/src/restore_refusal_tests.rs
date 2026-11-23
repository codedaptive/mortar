//! restore_refusal's own rules, with fakes only for the record decision, the state read and the state step: the
//! record's error passes through before any read or step; the record decider gets the op passed; the state is read
//! exactly once whatever the step; the read's StateRead and StateCorrupt pass through before any step; the step gets
//! the state read and the recorded state; StatePresent carries the path read; Write and AlreadyRestored are Ok. No
//! other file's decision runs here: ops and states are built as data.

use crate::join_state::JoinState;
use crate::restore_refusal::restore_refusal;
use crate::restore_step::RestoreStep;
use crate::JoinRunError;
use gitio::Oid;
use oplog::{OpPhase, OpRecord, OpResult, ValuedRecord};
use std::cell::{Cell, RefCell};
use std::path::PathBuf;

type Read = Result<(PathBuf, Option<JoinState>), JoinRunError>;

fn oid(digit: char) -> Oid {
    Oid::parse(&digit.to_string().repeat(40)).unwrap()
}

const STATE: &str = "/repo/.git/mortar-join.json";

fn op(op_id: &str) -> ValuedRecord {
    ValuedRecord {
        record: OpRecord {
            op_id: op_id.to_string(),
            key: format!("join-rebase:refs/heads/request/r1:{op_id}"),
            kind: "join-rebase".to_string(),
            actor: "42@host".to_string(),
            reason: "resolve".to_string(),
            at: 9,
            phase: OpPhase::Done,
            moves: vec![],
            result: Some(OpResult::Moved),
        },
        values: vec![],
    }
}

/// A paused state whose fields are all `digit`'s oid, so two digits give two different states.
fn state(digit: char) -> JoinState {
    JoinState {
        request_ref: "refs/heads/request/r1".to_string(),
        request_tip: oid(digit),
        onto: oid(digit),
        replayed_tip: oid(digit),
        commit: oid(digit),
        tree: oid(digit),
        remaining: vec![],
        files: vec![PathBuf::from("a.txt")],
        all_files: vec![PathBuf::from("a.txt")],
        files_written: true,
        rebased_to: None,
        folder_tree: None,
        base: None,
        base_before: None,
    }
}

fn recorded(_: &ValuedRecord) -> Result<JoinState, JoinRunError> {
    Ok(state('1'))
}

fn unread() -> Read {
    panic!("the state file was read after the record refused")
}

fn no_step(_: Option<&JoinState>, _: &JoinState) -> RestoreStep {
    panic!("the state step ran after a refusal")
}

fn found(existing: Option<JoinState>) -> impl FnOnce() -> Read {
    move || Ok((PathBuf::from(STATE), existing))
}

#[test]
fn not_a_resolve_op_from_the_record_passes_through_before_any_read_or_step() {
    let record = |_: &ValuedRecord| Err(JoinRunError::NotAResolveOp { op_id: "op-77".to_string() });
    match restore_refusal(&op("op-20"), record, unread, no_step) {
        Err(JoinRunError::NotAResolveOp { op_id }) => assert_eq!(op_id, "op-77"),
        other => panic!("expected NotAResolveOp, got {other:?}"),
    }
}

#[test]
fn recorded_state_corrupt_from_the_record_passes_through_before_any_read_or_step() {
    let record = |_: &ValuedRecord| {
        Err(JoinRunError::RecordedStateCorrupt { op_id: "op-77".to_string(), reason: "bad record".to_string() })
    };
    match restore_refusal(&op("op-20"), record, unread, no_step) {
        Err(JoinRunError::RecordedStateCorrupt { op_id, reason }) => {
            assert_eq!(op_id, "op-77");
            assert_eq!(reason, "bad record");
        }
        other => panic!("expected RecordedStateCorrupt, got {other:?}"),
    }
}

#[test]
fn the_record_decider_receives_the_op_passed() {
    let seen = RefCell::new(String::new());
    let record = |op: &ValuedRecord| {
        *seen.borrow_mut() = op.record.op_id.clone();
        Ok(state('1'))
    };
    restore_refusal(&op("op-31"), record, found(None), |_, _| RestoreStep::Write).unwrap();
    assert_eq!(*seen.borrow(), "op-31");
}

#[test]
fn the_state_file_is_read_exactly_once_for_each_step() {
    for step in [RestoreStep::Write, RestoreStep::AlreadyRestored, RestoreStep::StatePresent] {
        let calls = Cell::new(0);
        let read = || {
            calls.set(calls.get() + 1);
            found(None)()
        };
        let _ = restore_refusal(&op("op-20"), recorded, read, move |_, _| step);
        assert_eq!(calls.get(), 1);
    }
}

#[test]
fn a_state_read_error_passes_through_before_any_step() {
    let read = || -> Read {
        Err(JoinRunError::StateRead { path: PathBuf::from(STATE), source: std::io::Error::other("denied") })
    };
    match restore_refusal(&op("op-20"), recorded, read, no_step) {
        Err(JoinRunError::StateRead { path, source }) => {
            assert_eq!(path, PathBuf::from(STATE));
            assert_eq!(source.to_string(), "denied");
        }
        other => panic!("expected StateRead, got {other:?}"),
    }
}

#[test]
fn a_state_corrupt_error_passes_through_before_any_step() {
    let read = || -> Read { Err(JoinRunError::StateCorrupt { path: PathBuf::from(STATE), reason: "bad".to_string() }) };
    match restore_refusal(&op("op-20"), recorded, read, no_step) {
        Err(JoinRunError::StateCorrupt { path, reason }) => {
            assert_eq!(path, PathBuf::from(STATE));
            assert_eq!(reason, "bad");
        }
        other => panic!("expected StateCorrupt, got {other:?}"),
    }
}

#[test]
fn the_step_gets_the_state_read_as_existing_and_the_records_state_as_recorded() {
    for existing in [None, Some(state('2'))] {
        let seen = RefCell::new(None);
        let step = |existing: Option<&JoinState>, recorded: &JoinState| {
            *seen.borrow_mut() = Some((existing.cloned(), recorded.clone()));
            RestoreStep::Write
        };
        restore_refusal(&op("op-20"), recorded, found(existing.clone()), step).unwrap();
        assert_eq!(seen.into_inner(), Some((existing, state('1'))));
    }
}

#[test]
fn state_present_is_refused_at_the_path_read() {
    let read = || -> Read { Ok((PathBuf::from("/elsewhere/odd/state-file.json"), Some(state('2')))) };
    match restore_refusal(&op("op-20"), recorded, read, |_, _| RestoreStep::StatePresent) {
        Err(JoinRunError::StatePresent { state }) => assert_eq!(state, PathBuf::from("/elsewhere/odd/state-file.json")),
        other => panic!("expected StatePresent, got {other:?}"),
    }
}

#[test]
fn write_is_ok() {
    assert!(matches!(restore_refusal(&op("op-20"), recorded, found(None), |_, _| RestoreStep::Write), Ok(())));
}

#[test]
fn already_restored_is_ok() {
    let step = |_: Option<&JoinState>, _: &JoinState| RestoreStep::AlreadyRestored;
    assert!(matches!(restore_refusal(&op("op-20"), recorded, found(Some(state('1'))), step), Ok(())));
}
