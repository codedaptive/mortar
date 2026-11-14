//! backup_op: every field as a literal, and a key unique to each op id.

use super::backup_op;
use oplog::Op;

#[test]
fn names_the_host_in_its_key_and_reason_and_moves_nothing() {
    let op = backup_op("h1", "op-7".to_string(), "42@host", 1763030400);
    assert_eq!(
        op,
        Op {
            op_id: "op-7".to_string(),
            key: "land-backup:h1:op-7".to_string(),
            kind: "land-backup".to_string(),
            actor: "42@host".to_string(),
            reason: "backup pushes in-flight refs to refs/mortar-backup/h1/ on the remote".to_string(),
            at: 1763030400,
            moves: vec![],
        }
    );
}

#[test]
fn a_dotted_host_is_kept_whole() {
    let op = backup_op("box.local", "op-1".to_string(), "7@box", 5);
    assert_eq!(op.key, "land-backup:box.local:op-1");
    assert_eq!(op.reason, "backup pushes in-flight refs to refs/mortar-backup/box.local/ on the remote");
}

#[test]
fn calls_differing_only_in_op_id_have_different_keys() {
    let first = backup_op("h1", "op-1".to_string(), "1@a", 10);
    let second = backup_op("h1", "op-2".to_string(), "1@a", 10);
    assert_ne!(first.key, second.key);
}
