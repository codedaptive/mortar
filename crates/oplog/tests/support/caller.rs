use oplog::OpCaller;

pub fn caller(key: &str) -> OpCaller {
    OpCaller { key: key.into(), actor: "integration-requester".into(), reason: "explicit integration request".into() }
}
