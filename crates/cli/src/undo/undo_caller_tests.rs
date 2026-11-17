//! undo_caller: the oplog caller of an undo keys it by the target's op id, records the folder's actor, and gives the
//! reason "undo of <kind> op <op_id>" that reversed_op reads back (S-142 item 3).

use gitio::Oid;
use oplog::{OpCaller, OpPhase, OpRecord, OpResult, RefMove};

use super::undo_caller;

fn target(op_id: &str, kind: &str) -> OpRecord {
    OpRecord {
        op_id: op_id.to_string(),
        key: "save-key-1".to_string(),
        kind: kind.to_string(),
        actor: "orc/WO-cli-4".to_string(),
        reason: "fix the parser".to_string(),
        at: 40,
        phase: OpPhase::Done,
        moves: vec![RefMove {
            refname: "refs/heads/req/orc/WO-cli-4".to_string(),
            old: Some(Oid::parse(&"b".repeat(40)).expect("a valid oid")),
            new: Oid::parse(&"c".repeat(40)).expect("a valid oid"),
        }],
        result: Some(OpResult::Moved),
    }
}

#[test]
fn a_save_target_gives_its_key_actor_and_reason() {
    assert_eq!(
        undo_caller(&target("op-7", "save"), "orc/WO-cli-4"),
        OpCaller {
            key: "undo:op-7".to_string(),
            actor: "orc/WO-cli-4".to_string(),
            reason: "undo of save op op-7".to_string(),
        }
    );
}

#[test]
fn a_join_rebase_target_names_its_kind_and_op() {
    assert_eq!(
        undo_caller(&target("1700000000-ab12", "join-rebase"), "core/fix-1"),
        OpCaller {
            key: "undo:1700000000-ab12".to_string(),
            actor: "core/fix-1".to_string(),
            reason: "undo of join-rebase op 1700000000-ab12".to_string(),
        }
    );
}

#[test]
fn the_actor_is_the_one_given_not_the_targets() {
    assert_eq!(undo_caller(&target("op-7", "save"), "orc/other").actor, "orc/other");
}
