//! after_replay: a clean replay rebases from the request tip; a conflicted one pauses with every field kept, no
//! files written and no rebased tip.

use crate::after_replay::after_replay;
use crate::after_replay_step::AfterReplayStep;
use crate::join_state::JoinState;
use crate::ReplayResult;
use gitio::Oid;
use std::path::PathBuf;

const REQUEST_REF: &str = "refs/heads/request/r1";
const REQUEST_TIP: &str = "2222222222222222222222222222222222222222";
const ONTO: &str = "3333333333333333333333333333333333333333";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn clean_rebases_from_the_request_tip_to_the_new_tip() {
    let result = ReplayResult::Clean(oid("9999999999999999999999999999999999999999"));
    assert_eq!(
        after_replay(REQUEST_REF, &oid(REQUEST_TIP), &oid(ONTO), result),
        AfterReplayStep::Rebase {
            from: oid("2222222222222222222222222222222222222222"),
            to: oid("9999999999999999999999999999999999999999"),
        }
    );
}

#[test]
fn conflicted_pauses_with_every_field_carried_over() {
    let result = ReplayResult::Conflicted {
        commit: oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        replayed_tip: oid("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
        remaining: vec![oid("cccccccccccccccccccccccccccccccccccccccc"), oid("dddddddddddddddddddddddddddddddddddddddd")],
        tree: oid("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"),
        files: vec![PathBuf::from("src/a.rs"), PathBuf::from("b.txt")],
    };
    let AfterReplayStep::Pause(state) = after_replay(REQUEST_REF, &oid(REQUEST_TIP), &oid(ONTO), result) else {
        panic!("a conflicted replay must pause");
    };
    assert_eq!(
        state,
        JoinState {
            request_ref: "refs/heads/request/r1".to_string(),
            request_tip: oid("2222222222222222222222222222222222222222"),
            onto: oid("3333333333333333333333333333333333333333"),
            replayed_tip: oid("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
            commit: oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            tree: oid("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"),
            remaining: vec![
                oid("cccccccccccccccccccccccccccccccccccccccc"),
                oid("dddddddddddddddddddddddddddddddddddddddd"),
            ],
            files: vec![PathBuf::from("src/a.rs"), PathBuf::from("b.txt")],
            files_written: false,
            rebased_to: None,
        }
    );
}
