//! fast_forward_op: every field as a literal, and a key that differs with the op id.

use crate::fast_forward_op::fast_forward_op;
use gitio::Oid;
use oplog::{Op, RefMove};

const FROM: &str = "1111111111111111111111111111111111111111";
const TO: &str = "2222222222222222222222222222222222222222";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn every_field_is_set() {
    let op = fast_forward_op("refs/heads/zone/join", &oid(FROM), &oid(TO), "op-7".to_string(), "42@host", 1763030400);
    assert_eq!(
        op,
        Op {
            op_id: "op-7".to_string(),
            key: "join-fast-forward:refs/heads/zone/join:1111111111111111111111111111111111111111:\
                  2222222222222222222222222222222222222222:op-7"
                .to_string(),
            kind: "join-fast-forward".to_string(),
            actor: "42@host".to_string(),
            reason: "join fast-forwards refs/heads/zone/join from 1111111111111111111111111111111111111111 to \
                     2222222222222222222222222222222222222222"
                .to_string(),
            at: 1763030400,
            moves: vec![RefMove {
                refname: "refs/heads/zone/join".to_string(),
                old: Some(oid("1111111111111111111111111111111111111111")),
                new: oid("2222222222222222222222222222222222222222"),
            }],
        }
    );
}

#[test]
fn calls_differing_only_in_op_id_have_different_keys() {
    let first = fast_forward_op("refs/heads/zone/join", &oid(FROM), &oid(TO), "op-1".to_string(), "1@a", 10);
    let second = fast_forward_op("refs/heads/zone/join", &oid(FROM), &oid(TO), "op-2".to_string(), "1@a", 10);
    assert_ne!(first.key, second.key);
}
