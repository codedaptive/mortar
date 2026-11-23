//! zone_commit_op: kind commit-files, one move of the zone branch from its tip to the commit, and the given key,
//! actor, op id and time.

use gitio::Oid;
use oplog::{Op, RefMove};

use super::zone_commit_op;

const TIP: &str = "1111111111111111111111111111111111111111";
const COMMIT: &str = "2222222222222222222222222222222222222222";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn the_op_moves_the_zone_branch_from_tip_to_commit() {
    assert_eq!(
        zone_commit_op("core", &oid(TIP), &oid(COMMIT), "commit-files:k1", "42@host", "op-7".to_string(), 1700),
        Op {
            op_id: "op-7".to_string(),
            key: "commit-files:k1".to_string(),
            kind: "commit-files".to_string(),
            actor: "42@host".to_string(),
            reason: "commit-files moves refs/heads/zone/core from 1111111111111111111111111111111111111111 to \
                     2222222222222222222222222222222222222222"
                .to_string(),
            at: 1700,
            moves: vec![RefMove {
                refname: "refs/heads/zone/core".to_string(),
                old: Some(oid(TIP)),
                new: oid(COMMIT),
            }],
        }
    );
}
