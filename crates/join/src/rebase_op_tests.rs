//! rebase_op: every field as a literal, and a key that differs with the op id.

use crate::rebase_op::rebase_op;
use gitio::Oid;
use oplog::{Op, RefMove};

const FROM: &str = "1111111111111111111111111111111111111111";
const TO: &str = "2222222222222222222222222222222222222222";
const ONTO: &str = "3333333333333333333333333333333333333333";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn every_field_is_set() {
    let op = rebase_op(
        "refs/heads/request/r1",
        &oid(FROM),
        &oid(TO),
        &oid(ONTO),
        "op-7".to_string(),
        "42@host",
        1763030400,
    );
    assert_eq!(
        op,
        Op {
            op_id: "op-7".to_string(),
            key: "join-rebase:refs/heads/request/r1:1111111111111111111111111111111111111111:\
                  3333333333333333333333333333333333333333:op-7"
                .to_string(),
            kind: "join-rebase".to_string(),
            actor: "42@host".to_string(),
            reason: "join rebases refs/heads/request/r1 from 1111111111111111111111111111111111111111 to \
                     2222222222222222222222222222222222222222, replayed onto \
                     3333333333333333333333333333333333333333"
                .to_string(),
            at: 1763030400,
            moves: vec![RefMove {
                refname: "refs/heads/request/r1".to_string(),
                old: Some(oid("1111111111111111111111111111111111111111")),
                new: oid("2222222222222222222222222222222222222222"),
            }],
        }
    );
}

#[test]
fn calls_differing_only_in_op_id_have_different_keys() {
    let first = rebase_op("refs/heads/request/r1", &oid(FROM), &oid(TO), &oid(ONTO), "op-1".to_string(), "1@a", 10);
    let second = rebase_op("refs/heads/request/r1", &oid(FROM), &oid(TO), &oid(ONTO), "op-2".to_string(), "1@a", 10);
    assert_ne!(first.key, second.key);
}
