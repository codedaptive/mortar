//! zone_request_op: every field as a literal with both moves in order, the request move alone when the zone
//! is already at `to`, and a key that differs with the op id.

use crate::zone_request_op::zone_request_op;
use gitio::Oid;
use oplog::{Op, RefMove};

const ZONE_FROM: &str = "1111111111111111111111111111111111111111";
const REQUEST_FROM: &str = "2222222222222222222222222222222222222222";
const TO: &str = "3333333333333333333333333333333333333333";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn every_field_is_set_with_the_zone_move_first() {
    let op = zone_request_op(
        "refs/heads/zone/join",
        &oid(ZONE_FROM),
        "refs/heads/request/r1",
        &oid(REQUEST_FROM),
        &oid(TO),
        "op-7".to_string(),
        "42@host",
        1763030400,
    );
    assert_eq!(
        op,
        Op {
            op_id: "op-7".to_string(),
            key: "join-zone-request:refs/heads/zone/join:1111111111111111111111111111111111111111:\
                  refs/heads/request/r1:2222222222222222222222222222222222222222:\
                  3333333333333333333333333333333333333333:op-7"
                .to_string(),
            kind: "join-zone-request".to_string(),
            actor: "42@host".to_string(),
            reason: "join moves refs/heads/zone/join from 1111111111111111111111111111111111111111 and \
                     refs/heads/request/r1 from 2222222222222222222222222222222222222222 to \
                     3333333333333333333333333333333333333333"
                .to_string(),
            at: 1763030400,
            moves: vec![
                RefMove {
                    refname: "refs/heads/zone/join".to_string(),
                    old: Some(oid("1111111111111111111111111111111111111111")),
                    new: oid("3333333333333333333333333333333333333333"),
                },
                RefMove {
                    refname: "refs/heads/request/r1".to_string(),
                    old: Some(oid("2222222222222222222222222222222222222222")),
                    new: oid("3333333333333333333333333333333333333333"),
                },
            ],
        }
    );
}

#[test]
fn zone_already_at_to_moves_only_the_request() {
    let op = zone_request_op(
        "refs/heads/zone/join",
        &oid(TO),
        "refs/heads/request/r1",
        &oid(REQUEST_FROM),
        &oid(TO),
        "op-7".to_string(),
        "42@host",
        1763030400,
    );
    assert_eq!(
        op.moves,
        vec![RefMove {
            refname: "refs/heads/request/r1".to_string(),
            old: Some(oid("2222222222222222222222222222222222222222")),
            new: oid("3333333333333333333333333333333333333333"),
        }]
    );
}

#[test]
fn calls_differing_only_in_op_id_have_different_keys() {
    let op = |op_id: &str| {
        zone_request_op(
            "refs/heads/zone/join",
            &oid(ZONE_FROM),
            "refs/heads/request/r1",
            &oid(REQUEST_FROM),
            &oid(TO),
            op_id.to_string(),
            "1@a",
            10,
        )
    };
    assert_ne!(op("op-1").key, op("op-2").key);
}
