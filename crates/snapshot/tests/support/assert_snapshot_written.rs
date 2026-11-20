use super::Fixture;
use gitio::Oid;
use oplog::{read_op_log, OpPhase, OpResult, RefMove};
use snapshot::SnapshotOutcome;

pub fn assert_snapshot_written(f: &Fixture, saved: &SnapshotOutcome, head: &Oid) {
    assert!(saved.written);
    assert_eq!(f.git.read_ref(&f.refname).unwrap(), Some(saved.commit.clone()));
    assert_eq!(f.git.rev_parse("HEAD").unwrap(), Some(head.clone()));
    let info = f.git.commit_info(&saved.commit).unwrap();
    assert_eq!(info.tree, saved.tree);
    assert_eq!(info.parents, vec![head.clone()]);
    let log = read_op_log(&f.common).unwrap();
    assert!(log.torn_lines.is_empty());
    assert_eq!(log.records.len(), 2);
    assert_eq!(log.records[0].phase, OpPhase::Intent);
    assert_eq!(log.records[0].result, None);
    assert_eq!(log.records[1].phase, OpPhase::Done);
    assert_eq!(log.records[1].result, Some(OpResult::Moved));
    assert_eq!(log.records[0].op_id, log.records[1].op_id);
    for record in &log.records {
        assert_eq!(record.actor, "snapshot-caller");
        assert_eq!(record.kind, "snapshot");
        assert_eq!(record.moves, vec![RefMove { refname: f.refname.clone(), old: None, new: saved.commit.clone() }]);
    }
}
