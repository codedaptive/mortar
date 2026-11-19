//! Binary status/join -> real Oplog plain/valued compatibility, Git refs and Folders movement.
use crate::support::*;

#[test]
fn plain_join_rebase_reads_no_folder_tree_without_mutation_before_and_after_its_move() {
    let f = Fixture::new(); let (old, new, _zone) = f.pending_rebase();
    let records = oplog::read_valued_log(&f.common).unwrap().records;
    let rebase = records.iter().filter(|v| v.record.kind == "join-rebase").collect::<Vec<_>>();
    assert_eq!(rebase.len(), 2);
    assert!(rebase.iter().all(|v| v.values.is_empty()));
    let before = f.observe();
    let pending = f.last_move();
    let movement = pending.record.moves.iter().find(|m| m.refname == REQUEST).unwrap();
    assert_eq!(movement.old.as_ref(), Some(&old)); assert_eq!(movement.new, new);
    assert_eq!(pending.record.op_id, rebase[1].record.op_id); assert!(pending.values.is_empty());
    let status = f.status();
    assert_eq!(status.text("stalled_from"), old.as_str()); assert_eq!(status.text("stalled_to"), new.as_str());
    assert_eq!(status.text("next"), "mr join");
    assert_eq!(f.observe(), before);
    assert!(!f.request.join("src/upstream").exists());

    let report = f.report(&["--json", "join"], "join", 0, "joined");
    assert_eq!(report.text("finished_move_from"), old.as_str());
    assert_eq!(report.text("finished_move_to"), new.as_str()); assert!(!report.flag("saved_written"));
    // The command finishes the folder, then join fast-forwards the zone to the request.
    assert_eq!(f.git.read_ref(ZONE).unwrap(), Some(new.clone()));
    assert_eq!(f.blob(&new, "src/upstream"), b"upstream work\n");
    let after = f.observe();
    assert_eq!(f.last_move(), pending);
    let status = f.status(); assert!(status.is_null("stalled_from") && status.is_null("stalled_to"));
    assert_eq!(f.observe(), after); assert!(!f.lock_path().exists());
}
