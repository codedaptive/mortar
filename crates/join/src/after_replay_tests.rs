//! after_replay: a clean replay rebases from the request tip, earlier paths or not; a conflicted one pauses with every
//! field kept, all_files being the earlier rounds' paths then this round's new ones, no files written, no rebased tip
//! and no folder tree; its base is None when none is given and a copy of the given base otherwise.

use crate::after_replay::after_replay;
use crate::after_replay_step::AfterReplayStep;
use crate::join_state::JoinState;
use crate::partner::Partner;
use crate::request_base::RequestBase;
use crate::ReplayResult;
use gitio::Oid;
use std::path::PathBuf;

const REQUEST_REF: &str = "refs/heads/request/r1";
const REQUEST_TIP: &str = "2222222222222222222222222222222222222222";
const ONTO: &str = "3333333333333333333333333333333333333333";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn conflicted() -> ReplayResult {
    ReplayResult::Conflicted {
        commit: oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        replayed_tip: oid("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
        remaining: vec![oid("cccccccccccccccccccccccccccccccccccccccc"), oid("dddddddddddddddddddddddddddddddddddddddd")],
        tree: oid("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"),
        files: vec![PathBuf::from("src/a.rs"), PathBuf::from("b.txt")],
    }
}

#[test]
fn clean_rebases_from_the_request_tip_to_the_new_tip() {
    let result = ReplayResult::Clean(oid("9999999999999999999999999999999999999999"));
    assert_eq!(
        after_replay(REQUEST_REF, &oid(REQUEST_TIP), &oid(ONTO), &[PathBuf::from("old.txt")], None, result),
        AfterReplayStep::Rebase {
            from: oid("2222222222222222222222222222222222222222"),
            to: oid("9999999999999999999999999999999999999999"),
        }
    );
}

#[test]
fn conflicted_pauses_with_every_field_carried_over() {
    let AfterReplayStep::Pause(state) =
        after_replay(REQUEST_REF, &oid(REQUEST_TIP), &oid(ONTO), &[], None, conflicted())
    else {
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
            all_files: vec![PathBuf::from("src/a.rs"), PathBuf::from("b.txt")],
            files_written: false,
            rebased_to: None,
            folder_tree: None,
            base: None,
        }
    );
}

#[test]
fn a_later_round_pause_keeps_the_earlier_paths_then_adds_its_new_ones_once() {
    let earlier = [PathBuf::from("old/c.md"), PathBuf::from("b.txt")];
    let AfterReplayStep::Pause(state) =
        after_replay(REQUEST_REF, &oid(REQUEST_TIP), &oid(ONTO), &earlier, None, conflicted())
    else {
        panic!("a conflicted replay must pause");
    };
    assert_eq!(state.files, vec![PathBuf::from("src/a.rs"), PathBuf::from("b.txt")]);
    assert_eq!(state.all_files, vec![PathBuf::from("old/c.md"), PathBuf::from("b.txt"), PathBuf::from("src/a.rs")]);
    assert_eq!(state.folder_tree, None);
}

#[test]
fn a_pause_given_a_base_keeps_that_base_and_a_clean_replay_still_rebases() {
    let base = RequestBase {
        base: oid("4444444444444444444444444444444444444444"),
        partners: vec![Partner { zone: "docs".to_string(), tip: oid("5555555555555555555555555555555555555555") }],
    };
    let AfterReplayStep::Pause(state) =
        after_replay(REQUEST_REF, &oid(REQUEST_TIP), &oid(ONTO), &[], Some(&base), conflicted())
    else {
        panic!("a conflicted replay must pause");
    };
    assert_eq!(
        state.base,
        Some(RequestBase {
            base: oid("4444444444444444444444444444444444444444"),
            partners: vec![Partner {
                zone: "docs".to_string(),
                tip: oid("5555555555555555555555555555555555555555"),
            }],
        })
    );
    let clean = ReplayResult::Clean(oid("9999999999999999999999999999999999999999"));
    assert_eq!(
        after_replay(REQUEST_REF, &oid(REQUEST_TIP), &oid(ONTO), &[], Some(&base), clean),
        AfterReplayStep::Rebase {
            from: oid("2222222222222222222222222222222222222222"),
            to: oid("9999999999999999999999999999999999999999"),
        }
    );
}
