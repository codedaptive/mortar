//! ref_moved_from: a Stale result is RefMoved naming its ref and value; any other result is None.

use gitio::Oid;
use oplog::{OpPhase, OpRecord, OpResult, RefMove};

use super::ref_moved_from::ref_moved_from;
use super::FolderError;

const TIP: &str = "0123456789abcdef0123456789abcdef01234567";
const ZERO: &str = "0000000000000000000000000000000000000000";

fn record(phase: OpPhase, result: Option<OpResult>) -> OpRecord {
    OpRecord {
        op_id: "op-1".to_string(),
        key: "folders:remove:refs/heads/req/folders/WO-1@0123456789abcdef0123456789abcdef01234567".to_string(),
        kind: "folder-remove".to_string(),
        actor: "bilby".to_string(),
        reason: "remove folder branch refs/heads/req/folders/WO-1".to_string(),
        at: 7,
        phase,
        moves: vec![RefMove {
            refname: "refs/heads/req/folders/WO-1".to_string(),
            old: Some(Oid::parse(TIP).unwrap()),
            new: Oid::parse(ZERO).unwrap(),
        }],
        result,
    }
}

#[test]
fn stale_with_a_value_names_the_ref_and_where_it_is() {
    let stale = OpResult::Stale {
        refname: "refs/mortar/snap/folders/WO-1".to_string(),
        actual: Some(Oid::parse("2222222222222222222222222222222222222222").unwrap()),
    };
    match ref_moved_from(&record(OpPhase::Failed, Some(stale))) {
        Some(FolderError::RefMoved { refname, at }) => {
            assert_eq!(refname, "refs/mortar/snap/folders/WO-1");
            assert_eq!(at, Some(Oid::parse("2222222222222222222222222222222222222222").unwrap()));
        }
        other => panic!("expected RefMoved, got {other:?}"),
    }
}

#[test]
fn stale_with_no_value_says_the_ref_is_absent() {
    let stale = OpResult::Stale {
        refname: "refs/heads/req/folders/WO-1".to_string(),
        actual: None,
    };
    match ref_moved_from(&record(OpPhase::Failed, Some(stale))) {
        Some(FolderError::RefMoved { refname, at }) => {
            assert_eq!(refname, "refs/heads/req/folders/WO-1");
            assert_eq!(at, None);
        }
        other => panic!("expected RefMoved, got {other:?}"),
    }
}

#[test]
fn moved_is_none() {
    assert!(ref_moved_from(&record(OpPhase::Done, Some(OpResult::Moved))).is_none());
}

#[test]
fn no_result_is_none() {
    assert!(ref_moved_from(&record(OpPhase::Intent, None)).is_none());
}
