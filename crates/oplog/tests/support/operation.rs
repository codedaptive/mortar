use gitio::Oid;
use oplog::{Op, RefMove};

pub fn operation(objects: &[Oid; 3]) -> Op {
    Op {
        op_id: "integration-op-1".into(),
        key: "integration-repeat-key".into(),
        kind: "join".into(),
        actor: "integration-actor".into(),
        reason: "move two refs atomically".into(),
        at: 100,
        moves: ["refs/mortar/a", "refs/mortar/b"].map(|name| RefMove {
            refname: name.into(),
            old: Some(objects[0].clone()),
            new: objects[1].clone(),
        }).to_vec(),
    }
}
