//! Display of the restore refusals (NotAResolveOp, RecordedStateCorrupt, RestoreStale, StatePresent): each says why,
//! that nothing was written, and what the caller does next. Expected texts are written out by hand.

use crate::JoinRunError;
use gitio::Oid;
use std::path::PathBuf;

const OLD: &str = "1111111111111111111111111111111111111111";
const NEW: &str = "2222222222222222222222222222222222222222";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn not_a_resolve_op_names_the_op_and_says_to_undo_it_plainly_or_check_the_id() {
    let text = JoinRunError::NotAResolveOp { op_id: "op-12".to_string() }.to_string();
    assert_eq!(
        text,
        "op op-12 is not a resolve's final join-rebase op recording a paused join's state; nothing was written. \
         Only a resolve's final op restores a paused join: undo that op as a plain step, or check the op id with \
         `mr log`"
    );
}

#[test]
fn recorded_state_corrupt_names_the_op_the_reason_and_to_join_afresh() {
    let error = JoinRunError::RecordedStateCorrupt {
        op_id: "op-13".to_string(),
        reason: "expected true or false at byte 400".to_string(),
    };
    assert_eq!(
        error.to_string(),
        "the state recorded in op op-13 cannot be used: expected true or false at byte 400; nothing was written. \
         The op's ref move can still be undone, but the paused join cannot be put back from it: run `mr join` \
         to replay the request afresh"
    );
}

#[test]
fn restore_stale_names_the_ref_both_tips_and_to_undo_the_move_first() {
    let error = JoinRunError::RestoreStale {
        request_ref: "refs/heads/request/r1".to_string(),
        expected: oid(OLD),
        actual: Some(oid(NEW)),
    };
    assert_eq!(
        error.to_string(),
        "request ref refs/heads/request/r1 is at 2222222222222222222222222222222222222222, not at \
         1111111111111111111111111111111111111111 where the recorded paused join left it; nothing was written: move \
         refs/heads/request/r1 back to 1111111111111111111111111111111111111111 (undo the op's ref move) first, \
         then restore again"
    );
}

#[test]
fn restore_stale_with_the_ref_gone_says_it_does_not_exist() {
    let error = JoinRunError::RestoreStale {
        request_ref: "refs/heads/request/r1".to_string(),
        expected: oid(OLD),
        actual: None,
    };
    assert_eq!(
        error.to_string(),
        "request ref refs/heads/request/r1 does not exist, not at 1111111111111111111111111111111111111111 where the \
         recorded paused join left it; nothing was written: move refs/heads/request/r1 back to \
         1111111111111111111111111111111111111111 (undo the op's ref move) first, then restore again"
    );
}

#[test]
fn state_present_names_the_kept_file_and_the_two_ways_on() {
    let error = JoinRunError::StatePresent { state: PathBuf::from("/repo/.git/mortar-join.json") };
    assert_eq!(
        error.to_string(),
        "/repo/.git/mortar-join.json holds a different paused join; it was kept and nothing was written: finish \
         that join with `mr resolve`, or remove /repo/.git/mortar-join.json by hand, then restore again"
    );
}
