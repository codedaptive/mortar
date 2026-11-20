//! The sweep-refs op's every field, from literals; an empty list gives no moves.

use gitio::Oid;
use oplog::{Op, RefMove};

use super::sweep_refs_op;

const A: &str = "0123456789abcdef0123456789abcdef01234567";
const B: &str = "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn deletes_each_ref_from_where_it_was_read() {
    let refs = vec![
        ("refs/mortar/snap/folders/r1".to_string(), oid(A)),
        ("refs/mortar/snap/join/r2".to_string(), oid(B)),
    ];
    let op = sweep_refs_op("op-9".to_string(), "agent-a".to_string(), 1763030400, &refs).unwrap();
    assert_eq!(
        op,
        Op {
            op_id: "op-9".to_string(),
            key: "folders:sweep-refs:op-9".to_string(),
            kind: "folder-sweep-refs".to_string(),
            actor: "agent-a".to_string(),
            reason: "sweep 2 refs".to_string(),
            at: 1763030400,
            moves: vec![
                RefMove {
                    refname: "refs/mortar/snap/folders/r1".to_string(),
                    old: Some(oid(A)),
                    new: oid("0000000000000000000000000000000000000000"),
                },
                RefMove {
                    refname: "refs/mortar/snap/join/r2".to_string(),
                    old: Some(oid(B)),
                    new: oid("0000000000000000000000000000000000000000000000000000000000000000"),
                },
            ],
        }
    );
}

#[test]
fn an_empty_list_gives_no_moves() {
    let op = sweep_refs_op("op-1".to_string(), "agent-b".to_string(), 7, &[]).unwrap();
    assert_eq!(
        op,
        Op {
            op_id: "op-1".to_string(),
            key: "folders:sweep-refs:op-1".to_string(),
            kind: "folder-sweep-refs".to_string(),
            actor: "agent-b".to_string(),
            reason: "sweep 0 refs".to_string(),
            at: 7,
            moves: Vec::new(),
        }
    );
}
