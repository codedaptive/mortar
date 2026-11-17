//! S-148: public Join pre-check -> real Gitio state/ref I/O -> Oplog undo -> Join restore.
//! Each case owns Scenario's disposable repository and uses actual resolve records, never a private state codec.
use super::restore_support::{tree, Scenario, ACTOR, REQUEST, ZONE};
use join::{JoinAnswer, JoinRunError, PausedJoin};
use std::{fs, path::PathBuf};

fn check(r: &Scenario, op: &oplog::ValuedRecord) -> Result<(), JoinRunError> {
    r.call(|| join::check_restore_join_state(&r.f.git, &r.f.repo.path, op))
}

#[test]
fn precheck_then_real_undo_and_restore_recreates_the_exact_pause_and_resolves_the_same_tree() {
    let r = Scenario::paused(); let paused = r.state_bytes();
    let to = r.assert_rebased(r.resolve().unwrap()); let op = r.done(); r.assert_values(&op, &paused);
    let before = r.observe();
    check(&r, &op).unwrap();
    assert!(!r.state().exists()); assert_eq!(r.observe(), before);
    r.undo(&op);
    let before = r.observe();
    assert_eq!(r.restore(&op).unwrap(), r.restored(false));
    assert_eq!(r.state_bytes(), paused); assert_eq!(r.observe(), before);
    assert_eq!(r.assert_rebased(r.resolve().unwrap()), to);
}

#[test]
fn precheck_preserves_another_real_paused_join_and_refuses_before_undo() {
    let r = Scenario::paused(); r.assert_rebased(r.resolve().unwrap()); let op = r.done();
    let other_ref = "refs/heads/requests/other";
    let other_tree = tree(&r.f, &[("ok.txt", b"another request\n")]);
    let other_tip = r.f.git.commit_tree(&other_tree, std::slice::from_ref(&r.base), "other request\n", None, None).unwrap();
    r.f.git.run(&["update-ref", other_ref, other_tip.as_str()], &gitio::RunOpts::default()).unwrap();
    assert!(matches!(r.call(|| join::join(&r.f.git, &r.f.repo.path, ZONE, other_ref, ACTOR)).unwrap(),
        JoinAnswer::Conflicted { commit, files, .. } if commit == other_tip && files == vec![PathBuf::from("ok.txt")]));
    let paused = r.state_bytes(); let before = r.observe();
    assert!(matches!(check(&r, &op), Err(JoinRunError::StatePresent { state }) if state == r.state()));
    assert_eq!(r.state_bytes(), paused); assert_eq!(r.observe(), before);
    assert_eq!(join::paused_join(&r.common()).unwrap(), Some(PausedJoin {
        commit: other_tip, files: vec![PathBuf::from("ok.txt")],
    }));
    assert_eq!(r.f.git.read_ref(REQUEST).unwrap(), Some(op.record.moves[0].new.clone()));
}

#[test]
fn precheck_rejects_a_real_plain_join_record_without_writing_or_moving_anything() {
    let r = Scenario::new(false);
    assert!(matches!(r.join().unwrap(), JoinAnswer::Rebased { folder_tree: None, .. }));
    let op = r.done(); assert!(op.values.is_empty()); let before = r.observe();
    assert!(matches!(check(&r, &op), Err(JoinRunError::NotAResolveOp { op_id }) if op_id == op.record.op_id));
    assert!(!r.state().exists()); assert_eq!(r.observe(), before);
}

#[test]
fn precheck_accepts_its_own_already_restored_pause_and_keeps_every_byte() {
    let r = Scenario::paused(); let paused = r.state_bytes();
    r.assert_rebased(r.resolve().unwrap()); let op = r.done(); r.undo(&op);
    assert_eq!(r.restore(&op).unwrap(), r.restored(false));
    let before = r.observe();
    check(&r, &op).unwrap();
    assert_eq!(r.state_bytes(), paused); assert_eq!(r.observe(), before);
}

#[test]
fn precheck_at_the_new_request_tip_is_ok_not_restore_stale_before_any_undo() {
    let r = Scenario::paused(); let to = r.assert_rebased(r.resolve().unwrap()); let op = r.done();
    assert_ne!(to, r.tip); assert_eq!(r.f.git.read_ref(REQUEST).unwrap(), Some(to.clone()));
    let before = r.observe();
    check(&r, &op).unwrap();
    assert!(!r.state().exists()); assert_eq!(r.observe(), before);
    assert_eq!(r.f.git.read_ref(REQUEST).unwrap(), Some(to));
}

#[test]
fn precheck_keeps_a_corrupt_real_state_file_and_reports_state_corrupt_without_undo() {
    let r = Scenario::paused(); r.assert_rebased(r.resolve().unwrap()); let op = r.done();
    let corrupt = b"not a join state\n";
    fs::write(r.state(), corrupt).unwrap(); let before = r.observe();
    assert!(matches!(check(&r, &op), Err(JoinRunError::StateCorrupt { path, .. }) if path == r.state()));
    assert_eq!(r.state_bytes(), corrupt); assert_eq!(r.observe(), before);
}
