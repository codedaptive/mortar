//! records_op: its key, kind, reason and moves, and the op id, actor and time it carries as given.

use super::records_op;
use gitio::Oid;
use oplog::{Op, RefMove};

const ZONE_REF: &str = "refs/heads/zone/land";
const RECORD_REF: &str = "refs/mortar/replayed/land/1111111111111111111111111111111111111111";
const VALUE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const ZERO: &str = "0000000000000000000000000000000000000000";

/// The one helper building values only a constructor makes (Oid::parse).
fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn delete() -> RefMove {
    RefMove { refname: RECORD_REF.to_string(), old: Some(oid(VALUE)), new: oid(ZERO) }
}

#[test]
fn the_op_moves_only_the_records_under_the_after_publish_kind() {
    assert_eq!(
        records_op(ZONE_REF, vec![delete()], "op-7".to_string(), "agent-a", 1_700_000_000),
        Op {
            op_id: "op-7".to_string(),
            key: "after-publish-records:refs/heads/zone/land:op-7".to_string(),
            kind: "after-publish".to_string(),
            actor: "agent-a".to_string(),
            reason: "after-publish deletes replay records of refs/heads/zone/land".to_string(),
            at: 1_700_000_000,
            moves: vec![delete()],
        }
    );
}

#[test]
fn each_op_id_is_its_own_key() {
    let first = records_op(ZONE_REF, vec![delete()], "op-1".to_string(), "agent-a", 1);
    let second = records_op(ZONE_REF, vec![delete()], "op-2".to_string(), "agent-a", 1);
    assert_eq!(first.key, "after-publish-records:refs/heads/zone/land:op-1");
    assert_eq!(second.key, "after-publish-records:refs/heads/zone/land:op-2");
}
