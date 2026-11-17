//! find_pending_move where the last mover is an undo of a join-rebase and the reversed op's Done record decides: an
//! undo whose reversed op's Done record carries join_state (a resolve step, S-148) is none; one whose reversed op
//! carries folder_tree only, has no Done record in the log, or only failed, is the move back (cause Undo, S-143).
//! ZIT (S-102, rule 11): find_pending_move reads the reversed op through undo_reason's reversed_op, done_record and
//! is_resolve_step, run for real.

use gitio::Oid;
use oplog::{OpPhase, OpRecord, RefMove, ValuedRecord};

use cli::join::{find_pending_move, MoveCause, PendingMove};

const REQUEST: &str = "refs/heads/req/core/fix-1";
const OLD: &str = "1111111111111111111111111111111111111111";
const NEW: &str = "2222222222222222222222222222222222222222";
const TREE: &str = "4444444444444444444444444444444444444444";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn record(op_id: &str, kind: &str, phase: OpPhase, refname: &str, old: Option<&str>, new: &str) -> ValuedRecord {
    ValuedRecord {
        record: OpRecord {
            op_id: op_id.to_string(),
            key: format!("{kind}:{op_id}"),
            kind: kind.to_string(),
            actor: "agent-a".to_string(),
            reason: "test".to_string(),
            at: 1_700_000_000,
            phase,
            moves: vec![RefMove { refname: refname.to_string(), old: old.map(oid), new: oid(new) }],
            result: None,
        },
        values: Vec::new(),
    }
}

fn with_values(mut valued: ValuedRecord, values: &[(&str, &str)]) -> ValuedRecord {
    valued.values = values.iter().map(|(name, value)| (name.to_string(), value.to_string())).collect();
    valued
}

const UNDO_OF_REBASE: &str = "undo of join-rebase op op-5";

/// The join-rebase op-5 moved the request from OLD to the rebased tip NEW.
fn rebase_5() -> ValuedRecord {
    record("op-5", "join-rebase", OpPhase::Done, REQUEST, Some(OLD), NEW)
}

/// An undo op whose move of the request ref goes from `old` back to OLD, with `reason`.
fn undo(reason: &str, old: Option<&str>) -> ValuedRecord {
    let mut valued = record("op-undo", "undo", OpPhase::Done, REQUEST, old, OLD);
    valued.record.reason = reason.to_string();
    valued
}

fn undone() -> PendingMove {
    PendingMove {
        request_ref: REQUEST.to_string(),
        old: oid(NEW),
        new: oid(OLD),
        op_id: "op-undo".to_string(),
        folder_tree: None,
        cause: MoveCause::Undo,
    }
}

/// The resolve's final join-rebase op-5: the move from OLD to the rebased tip NEW, carrying "join_state".
fn resolve_5() -> ValuedRecord {
    with_values(rebase_5(), &[("folder_tree", TREE), ("join_state", "{\"paused\":true}")])
}

#[test]
fn an_undo_of_a_resolve_step_is_none() {
    let records = [resolve_5(), undo(UNDO_OF_REBASE, Some(NEW))];
    assert_eq!(find_pending_move(&records, REQUEST, Some(&oid(OLD))), None);
}

#[test]
fn an_undo_of_a_plain_join_rebase_carrying_folder_tree_only_is_the_move_back() {
    let records = [with_values(rebase_5(), &[("folder_tree", TREE)]), undo(UNDO_OF_REBASE, Some(NEW))];
    assert_eq!(find_pending_move(&records, REQUEST, Some(&oid(OLD))), Some(undone()));
}

#[test]
fn an_undo_whose_reversed_op_is_missing_from_the_log_is_the_move_back() {
    let records = [undo(UNDO_OF_REBASE, Some(NEW))];
    assert_eq!(find_pending_move(&records, REQUEST, Some(&oid(OLD))), Some(undone()));
}

#[test]
fn an_undo_whose_reversed_resolve_step_only_failed_is_the_move_back() {
    let failed = with_values(
        record("op-5", "join-rebase", OpPhase::Failed, REQUEST, Some(OLD), NEW),
        &[("join_state", "{}")],
    );
    let records = [rebase_5(), failed, undo(UNDO_OF_REBASE, Some(NEW))];
    assert_eq!(find_pending_move(&records, REQUEST, Some(&oid(OLD))), Some(undone()));
}
