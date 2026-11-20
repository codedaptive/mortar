//! The oplog op that deletes every attempt branch of a request.

use gitio::{GitError, Oid};
use oplog::{Op, RefMove};

use super::Dropped;

/// Kind "folder-drop-attempts", key "folders:drop-attempts:<request_ref>:<op_id>", reason "drop <n> attempts of
/// <request_ref>", n being the number of branches in `dropped.delete`. Each drop is planned afresh from the refs, so
/// its key is unique and oplog::apply never answers it Repeated. One move per `dropped.delete` (refname, oid), in
/// order: old Some(oid), so the op moves nothing unless every branch is still where it was read, and new all zeros
/// of the oid's length, which deletes the ref (oplog RefMove). The request ref is not moved. The caller supplies
/// `op_id` and `at` (unix seconds), so this stays pure. The Err is Oid::parse's refusal of the zeros, which zeros of
/// an Oid's own length never get.
pub fn drop_op(op_id: String, actor: String, at: u64, request_ref: &str, dropped: &Dropped) -> Result<Op, GitError> {
    let moves = dropped
        .delete
        .iter()
        .map(|(refname, oid)| {
            Ok(RefMove {
                refname: refname.clone(),
                old: Some(oid.clone()),
                new: Oid::parse(&"0".repeat(oid.as_str().len()))?,
            })
        })
        .collect::<Result<Vec<_>, GitError>>()?;
    Ok(Op {
        key: format!("folders:drop-attempts:{request_ref}:{op_id}"),
        op_id,
        kind: "folder-drop-attempts".to_string(),
        actor,
        reason: format!("drop {} attempts of {request_ref}", dropped.delete.len()),
        at,
        moves,
    })
}
