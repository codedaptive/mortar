//! The oplog op that deletes the refs a sweep picks.

use gitio::{GitError, Oid};
use oplog::{Op, RefMove};

/// Kind "folder-sweep-refs", key "folders:sweep-refs:<op_id>", reason "sweep <n> refs": each sweep is decided
/// afresh from the refs, so its key is unique and oplog::apply_protected never answers it Repeated. One move per
/// ref in `refs` (refname, id: the refs sweep_pick picks), in order: old Some(its id), so the op moves nothing
/// unless every ref is still where it was read, and new all zeros of the id's length, which deletes the ref (oplog
/// RefMove). The caller supplies `op_id` and `at` (unix seconds), so this stays pure. The Err is Oid::parse's
/// refusal of the zeros, which zeros of an Oid's own length never get.
pub fn sweep_refs_op(op_id: String, actor: String, at: u64, refs: &[(String, Oid)]) -> Result<Op, GitError> {
    let moves = refs
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
        key: format!("folders:sweep-refs:{op_id}"),
        op_id,
        kind: "folder-sweep-refs".to_string(),
        actor,
        reason: format!("sweep {} refs", refs.len()),
        at,
        moves,
    })
}
