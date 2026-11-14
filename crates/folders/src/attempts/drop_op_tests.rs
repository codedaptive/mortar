//! The drop op's every field, from literals: each attempt branch deleted by zeros of its own length, the request
//! ref never moved, the key ending in the op id.

use std::path::PathBuf;

use gitio::Oid;
use oplog::{Op, RefMove};

use super::{drop_op, Dropped};

const T1: &str = "1111111111111111111111111111111111111111";
const T2: &str = "2222222222222222222222222222222222222222";
const ZERO: &str = "0000000000000000000000000000000000000000";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn deletes_each_attempt_branch_and_leaves_the_request_ref() {
    let dropped = Dropped {
        remove: vec![PathBuf::from("/t/try-1"), PathBuf::from("/t/try-2")],
        delete: vec![
            ("refs/heads/try/folders/WO-6/1".to_string(), oid(T1)),
            ("refs/heads/try/folders/WO-6/2".to_string(), oid(T2)),
        ],
    };
    let op = drop_op("op-1".to_string(), "bilby".to_string(), 1763030400, "refs/heads/req/folders/WO-6", &dropped).unwrap();
    assert_eq!(
        op,
        Op {
            op_id: "op-1".to_string(),
            key: "folders:drop-attempts:refs/heads/req/folders/WO-6:op-1".to_string(),
            kind: "folder-drop-attempts".to_string(),
            actor: "bilby".to_string(),
            reason: "drop 2 attempts of refs/heads/req/folders/WO-6".to_string(),
            at: 1763030400,
            moves: vec![
                RefMove {
                    refname: "refs/heads/try/folders/WO-6/1".to_string(),
                    old: Some(oid(T1)),
                    new: oid(ZERO),
                },
                RefMove {
                    refname: "refs/heads/try/folders/WO-6/2".to_string(),
                    old: Some(oid(T2)),
                    new: oid(ZERO),
                },
            ],
        }
    );
}

#[test]
fn two_drops_of_the_same_request_have_different_keys() {
    let dropped = Dropped {
        remove: Vec::new(),
        delete: vec![("refs/heads/try/a/b/1".to_string(), oid(T1))],
    };
    let first = drop_op("op-a".to_string(), "simms".to_string(), 7, "refs/heads/req/a/b", &dropped).unwrap();
    let second = drop_op("op-b".to_string(), "simms".to_string(), 7, "refs/heads/req/a/b", &dropped).unwrap();
    assert_eq!(first.key, "folders:drop-attempts:refs/heads/req/a/b:op-a");
    assert_eq!(second.key, "folders:drop-attempts:refs/heads/req/a/b:op-b");
}

#[test]
fn a_sha256_attempt_branch_is_deleted_by_64_zeros() {
    let tip = oid("abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789");
    let dropped = Dropped {
        remove: Vec::new(),
        delete: vec![("refs/heads/try/a/b/1".to_string(), tip)],
    };
    let op = drop_op("op-2".to_string(), "simms".to_string(), 7, "refs/heads/req/a/b", &dropped).unwrap();
    assert_eq!(op.moves[0].new, oid("0000000000000000000000000000000000000000000000000000000000000000"));
}
