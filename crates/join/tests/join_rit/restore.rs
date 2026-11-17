//! WO-join-rit-9 / S-142(1): public Join resolve -> Oplog valued log/undo -> Join restore -> resolve.
//! Five real-repository cases; the sixth is restore_crash's actual killed resolve. No private state codec.

use super::restore_support::{tree, Scenario, ACTOR, REQUEST, ZONE};
use join::{JoinAnswer, JoinRunError, PausedJoin};
use std::{fs, path::PathBuf};

#[test]
fn restore_after_real_resolve_and_oplog_undo_restores_exact_pause_then_resolves_the_same_tree() {
    let r = Scenario::paused(); let paused = r.state_bytes();
    let to = r.assert_rebased(r.resolve().unwrap()); let op = r.done();
    r.assert_values(&op, &paused); r.undo(&op);
    assert!(!r.state().exists()); let before = r.observe();
    assert_eq!(r.restore(&op).unwrap(), r.restored(false));
    assert_eq!(r.state_bytes(), paused, "exact pre-final-resolve bytes, including the single newline");
    assert_eq!(r.observe(), before, "restore writes only the paused state, not refs, index, log or local edits");
    assert_eq!(r.assert_rebased(r.resolve().unwrap()), to);
    let records = oplog::read_op_log(&r.common()).unwrap().records;
    assert_eq!(records.len(), 6, "resolve, undo, resolve: each exactly one intent/done pair");
    assert_eq!(records.iter().filter(|v| v.kind == "join-rebase").count(), 4);
    assert_eq!(records.iter().filter(|v| v.kind == "undo").count(), 2);
}

#[test]
fn restore_repeat_answers_already_restored_without_changing_state_refs_log_index_or_files() {
    let r = Scenario::paused(); let paused = r.state_bytes();
    r.assert_rebased(r.resolve().unwrap()); let op = r.done(); r.undo(&op);
    assert_eq!(r.restore(&op).unwrap(), r.restored(false));
    let before = r.observe();
    assert_eq!(r.restore(&op).unwrap(), r.restored(true));
    assert_eq!(r.state_bytes(), paused); assert_eq!(r.observe(), before);
}

#[test]
fn restore_preserves_another_real_paused_join_and_returns_state_present() {
    let r = Scenario::paused(); r.assert_rebased(r.resolve().unwrap()); let op = r.done(); r.undo(&op);
    let other_ref = "refs/heads/requests/other";
    let other_tree = tree(&r.f, &[("ok.txt", b"another request\n")]);
    let other_tip = r.f.git.commit_tree(&other_tree, std::slice::from_ref(&r.base), "other request\n", None, None).unwrap();
    r.f.git.run(&["update-ref", other_ref, other_tip.as_str()], &gitio::RunOpts::default()).unwrap();
    assert!(matches!(r.call(|| join::join(&r.f.git, &r.f.repo.path, ZONE, other_ref, ACTOR)).unwrap(),
        JoinAnswer::Conflicted { commit, files, .. } if commit == other_tip && files == vec![PathBuf::from("ok.txt")]));
    let paused = r.state_bytes(); let before = r.observe();
    assert!(matches!(r.restore(&op), Err(JoinRunError::StatePresent { state }) if state == r.state()));
    assert_eq!(r.state_bytes(), paused); assert_eq!(r.observe(), before);
    assert_eq!(join::paused_join(&r.common()).unwrap(), Some(PausedJoin {
        commit: other_tip, files: vec![PathBuf::from("ok.txt")],
    }));
}

#[test]
fn restore_before_undo_refuses_at_the_new_tip_without_creating_state_or_changing_anything() {
    let r = Scenario::paused(); let to = r.assert_rebased(r.resolve().unwrap()); let op = r.done();
    let before = r.observe();
    assert!(matches!(r.restore(&op), Err(JoinRunError::RestoreStale { request_ref, expected, actual })
        if request_ref == REQUEST && expected == r.tip && actual == Some(to)));
    assert!(!r.state().exists()); assert_eq!(r.observe(), before);
}

#[test]
fn restore_rejects_a_real_plain_join_rebase_done_record_without_writing_state() {
    let r = Scenario::new(false);
    let answer = r.join().unwrap();
    assert!(matches!(answer, JoinAnswer::Rebased { request_ref, from, to, folder_tree: None }
        if request_ref == REQUEST && from == r.tip && to != r.tip));
    let op = r.done(); assert!(op.values.is_empty(), "plain join does not record a resolve pause");
    let before = r.observe();
    assert!(matches!(r.restore(&op), Err(JoinRunError::NotAResolveOp { op_id }) if op_id == op.record.op_id));
    assert!(!r.state().exists()); assert_eq!(r.observe(), before);
    assert_eq!(fs::read(r.f.repo.path.join("ok.txt")).unwrap(), b"keep ok.txt\n");
}
