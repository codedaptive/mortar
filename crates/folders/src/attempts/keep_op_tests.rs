//! The keep op's every field, from literals: the request branch moved to the kept tip, then each attempt branch
//! deleted by zeros of its own length.

use std::path::PathBuf;

use gitio::Oid;
use oplog::{Op, RefMove};

use super::{keep_op, Kept};

const REQ: &str = "0123456789abcdef0123456789abcdef01234567";
const T1: &str = "1111111111111111111111111111111111111111";
const T2: &str = "2222222222222222222222222222222222222222";
const ZERO: &str = "0000000000000000000000000000000000000000";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn moves_the_request_branch_then_deletes_each_attempt_branch() {
    let kept = Kept {
        tip: oid(T2),
        remove: vec![PathBuf::from("/t/try-1"), PathBuf::from("/t/try-2")],
        delete: vec![
            ("refs/heads/try/folders/r6/1".to_string(), oid(T1)),
            ("refs/heads/try/folders/r6/2".to_string(), oid(T2)),
        ],
    };
    let op = keep_op("op-1".to_string(), "agent-a".to_string(), 1763030400, "refs/heads/req/folders/r6", &oid(REQ), &kept)
        .unwrap();
    assert_eq!(
        op,
        Op {
            op_id: "op-1".to_string(),
            key: "folders:keep:refs/heads/req/folders/r6@2222222222222222222222222222222222222222".to_string(),
            kind: "folder-keep".to_string(),
            actor: "agent-a".to_string(),
            reason: "keep 2222222222222222222222222222222222222222 on refs/heads/req/folders/r6".to_string(),
            at: 1763030400,
            moves: vec![
                RefMove {
                    refname: "refs/heads/req/folders/r6".to_string(),
                    old: Some(oid(REQ)),
                    new: oid(T2),
                },
                RefMove {
                    refname: "refs/heads/try/folders/r6/1".to_string(),
                    old: Some(oid(T1)),
                    new: oid(ZERO),
                },
                RefMove {
                    refname: "refs/heads/try/folders/r6/2".to_string(),
                    old: Some(oid(T2)),
                    new: oid(ZERO),
                },
            ],
        }
    );
}

#[test]
fn a_sha256_attempt_branch_is_deleted_by_64_zeros() {
    let tip = oid("abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789");
    let kept = Kept {
        tip: tip.clone(),
        remove: Vec::new(),
        delete: vec![("refs/heads/try/a/b/1".to_string(), tip)],
    };
    let request = oid("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");
    let op = keep_op("op-2".to_string(), "agent-b".to_string(), 7, "refs/heads/req/a/b", &request, &kept).unwrap();
    assert_eq!(op.moves[1].new, oid("0000000000000000000000000000000000000000000000000000000000000000"));
}
