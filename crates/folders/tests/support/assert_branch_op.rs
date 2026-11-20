use gitio::Oid;
use oplog::{OpPhase, OpRecord, OpResult, RefMove};

pub fn assert_branch_op(pair: &[OpRecord], branch: &str, tip: &Oid) {
    assert_eq!(pair.len(), 2);
    assert_eq!(pair[0].phase, OpPhase::Intent);
    assert_eq!(pair[0].result, None);
    assert_eq!(pair[1].phase, OpPhase::Done);
    assert_eq!(pair[1].result, Some(OpResult::Moved));
    assert_eq!(pair[0].op_id, pair[1].op_id);
    for record in pair {
        assert!(!record.op_id.is_empty());
        assert_eq!(record.actor, "folder-integration-caller");
        assert_eq!(record.kind, "folder-branch");
        assert_eq!(record.reason, format!("make folder branch {branch}"));
        assert_eq!(record.key, format!("folders:branch:{branch}@{tip}"));
        assert_eq!(record.moves, vec![RefMove { refname: format!("refs/heads/{branch}"), old: None, new: tip.clone() }]);
    }
}
