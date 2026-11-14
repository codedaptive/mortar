//! squash_op: every field as a literal, for a first squash and a remade one, and a key unique to each op id.

use super::squash_op;
use gitio::Oid;
use oplog::{Op, RefMove};

const OLD: &str = "1111111111111111111111111111111111111111";
const NEW: &str = "2222222222222222222222222222222222222222";

/// The one helper building values only a constructor makes (Oid::parse).
fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn a_remade_squash_moves_the_ref_from_the_prior_commit() {
    let old = oid(OLD);
    let op = squash_op("refs/mortar/squash/land", Some(&old), &oid(NEW), "op-7".to_string(), "42@host", 1763030400);
    assert_eq!(
        op,
        Op {
            op_id: "op-7".to_string(),
            key: "land-squash:refs/mortar/squash/land:1111111111111111111111111111111111111111:\
                  2222222222222222222222222222222222222222:op-7"
                .to_string(),
            kind: "land-squash".to_string(),
            actor: "42@host".to_string(),
            reason: "squash moves refs/mortar/squash/land from 1111111111111111111111111111111111111111 to \
                     2222222222222222222222222222222222222222"
                .to_string(),
            at: 1763030400,
            moves: vec![RefMove {
                refname: "refs/mortar/squash/land".to_string(),
                old: Some(oid("1111111111111111111111111111111111111111")),
                new: oid("2222222222222222222222222222222222222222"),
            }],
        }
    );
}

#[test]
fn a_first_squash_creates_the_ref() {
    let op = squash_op("refs/mortar/squash/land", None, &oid(NEW), "op-1".to_string(), "7@box", 5);
    assert_eq!(
        op,
        Op {
            op_id: "op-1".to_string(),
            key: "land-squash:refs/mortar/squash/land:none:2222222222222222222222222222222222222222:op-1".to_string(),
            kind: "land-squash".to_string(),
            actor: "7@box".to_string(),
            reason: "squash moves refs/mortar/squash/land from nothing to 2222222222222222222222222222222222222222"
                .to_string(),
            at: 5,
            moves: vec![RefMove {
                refname: "refs/mortar/squash/land".to_string(),
                old: None,
                new: oid("2222222222222222222222222222222222222222"),
            }],
        }
    );
}

#[test]
fn calls_differing_only_in_op_id_have_different_keys() {
    let first = squash_op("refs/mortar/squash/land", None, &oid(NEW), "op-1".to_string(), "1@a", 10);
    let second = squash_op("refs/mortar/squash/land", None, &oid(NEW), "op-2".to_string(), "1@a", 10);
    assert_ne!(first.key, second.key);
}
