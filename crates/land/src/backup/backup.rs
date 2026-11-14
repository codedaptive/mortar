//! backup: this host's in-flight refs mirrored to `refs/mortar-backup/<host>/` on the remote (SPEC-land 08). It
//! takes no lock: the caller holds `land` and has run oplog recover (S-16), so no recover races its open intent.
//! It never touches the remote's origin or integration branch, nor any ref outside the host's backup namespace.
//! No unit test: it runs git and writes the op log (S-5); plan_backup_tests, backup_answer_tests and
//! backup_op_tests cover its decisions. tests/backup_zit.rs covers it in a throwaway repository with a bare remote:
//! a_backup_restores_every_ref_into_a_fresh_store, a_second_backup_pushes_nothing,
//! a_ref_deleted_locally_is_deleted_from_the_backup, refs_outside_the_four_prefixes_are_never_backed_up and
//! a_pushing_backup_logs_an_intent_then_done_and_an_idle_one_logs_nothing.

use crate::backup::{backup_answer, backup_op, backup_prefix, list_backed_up_refs, plan_backup, Backup, BackupError};
use crate::squash::LandRefs;
use gitio::{Git, PushRef};
use oplog::{append_op_record, close_record, host_name, intent_record, new_op_id, op_git, unix_now, OpPhase, OpResult};

/// Pushes every local ref under BACKUP_PREFIXES to its backup ref under `refs/mortar-backup/<host>/` on
/// `refs.remote`, `host` being this machine's name (Host when it cannot be read, BadHost when it cannot name a
/// namespace). Backups already equal are not pushed; a backup whose local ref is gone is deleted; each set or
/// delete is leased at the backup's listed value, so it moves whatever its history, and one moved meanwhile on
/// the remote is refused (Refused, naming the first). Nothing to push: Backup with no intent written and no push
/// run. Otherwise the push runs inside a zero-move op of kind land-backup, with MORTAR_OP set to its id: its
/// intent first, then a Done/Moved close once the push answered (refusals included), or a Failed/NotMoved close
/// and the Git error when the push itself failed.
pub fn backup(git: &Git, refs: &LandRefs, actor: &str) -> Result<Backup, BackupError> {
    let host = host_name().map_err(BackupError::Host)?;
    let prefix = backup_prefix(&host)?;
    let local = list_backed_up_refs(git)?;
    let remote = git.ls_remote(&refs.remote, &[]).map_err(BackupError::Git)?;
    let plan = plan_backup(&prefix, &local, &remote);
    if plan.push.is_empty() {
        return Ok(Backup { pushed: Vec::new(), unchanged: plan.unchanged });
    }
    let common = git.common_dir().map_err(BackupError::Git)?;
    let op = backup_op(&host, new_op_id(), actor, unix_now());
    let intent = intent_record(&op);
    append_op_record(&common, &intent).map_err(BackupError::Oplog)?;
    let push_refs: Vec<PushRef> = plan.push.iter().map(|(_, push_ref)| push_ref.clone()).collect();
    match op_git(git, &op.op_id).push(&refs.remote, &push_refs) {
        Ok(outcomes) => {
            let done = close_record(&intent, OpPhase::Done, OpResult::Moved, unix_now());
            append_op_record(&common, &done).map_err(BackupError::Oplog)?;
            backup_answer(plan, outcomes)
        }
        Err(e) => {
            let failed = close_record(&intent, OpPhase::Failed, OpResult::NotMoved { refs: Vec::new() }, unix_now());
            append_op_record(&common, &failed).map_err(BackupError::Oplog)?;
            Err(BackupError::Git(e))
        }
    }
}
