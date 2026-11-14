//! restore_op: every field as a literal for two created refs, and a key unique to each op id.

use super::restore_op;
use gitio::Oid;
use oplog::{Op, RefMove};

const A: &str = "1111111111111111111111111111111111111111";
const B: &str = "2222222222222222222222222222222222222222";

/// The one helper building values only a constructor makes (Oid::parse).
fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn creates_each_ref_from_absent_in_order() {
    let created = [("refs/heads/zone/b".to_string(), oid(B)), ("refs/mortar/snap/s1".to_string(), oid(A))];
    let op = restore_op("h1", &created, "op-7".to_string(), "42@host", 1763030400);
    assert_eq!(
        op,
        Op {
            op_id: "op-7".to_string(),
            key: "land-restore-backup:h1:op-7".to_string(),
            kind: "land-restore-backup".to_string(),
            actor: "42@host".to_string(),
            reason: "restore_backup creates 2 local refs from host h1's backup".to_string(),
            at: 1763030400,
            moves: vec![
                RefMove {
                    refname: "refs/heads/zone/b".to_string(),
                    old: None,
                    new: oid("2222222222222222222222222222222222222222"),
                },
                RefMove {
                    refname: "refs/mortar/snap/s1".to_string(),
                    old: None,
                    new: oid("1111111111111111111111111111111111111111"),
                },
            ],
        }
    );
}

#[test]
fn the_reason_counts_one_ref_and_names_the_host() {
    let created = [("refs/heads/zone/a".to_string(), oid(A))];
    let op = restore_op("box.local", &created, "op-1".to_string(), "7@box", 5);
    assert_eq!(op.reason, "restore_backup creates 1 local ref from host box.local's backup");
    assert_eq!(op.key, "land-restore-backup:box.local:op-1");
}

#[test]
fn calls_differing_only_in_op_id_have_different_keys() {
    let created = [("refs/heads/zone/a".to_string(), oid(A))];
    let first = restore_op("h1", &created, "op-1".to_string(), "1@a", 10);
    let second = restore_op("h1", &created, "op-2".to_string(), "1@a", 10);
    assert_ne!(first.key, second.key);
}
