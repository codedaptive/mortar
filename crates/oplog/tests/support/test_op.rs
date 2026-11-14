use oplog::{Op, RefMove};

/// Gives each real transaction a distinct, explicit identity and actor for history assertions.
pub fn test_op(id: &str, actor: &str, moves: Vec<RefMove>) -> Op {
    Op {
        op_id: id.into(), key: format!("key:{id}"), kind: "join".into(),
        actor: actor.into(), reason: "integration transaction".into(), at: 100, moves,
    }
}
