//! restore_backup: a host's backup on the remote brought back into the local store (SPEC-land 08). The actor is
//! forced by oplog's Op.actor (crates/oplog/src/store/op.rs): the op that creates the refs needs one; SPEC-land
//! 08-backup.md names it in restore_backup's signature.
//! No unit test: it runs git and writes the op log (S-5); restored_after_fetch_tests, restore_op_tests and
//! restore_applied_tests cover its decisions, and the backup integration tests (backup_zit) cover the run and
//! restore_plan.

use crate::backup::{
    backup_prefix, list_backed_up_refs, restore_applied, restore_op, restore_plan, restored_after_fetch,
    BackupError, Restored,
};
use crate::squash::LandRefs;
use gitio::{FetchRefOutcome, Git, Oid};
use oplog::{apply, new_op_id, unix_now};

/// Restores `host`'s backup (`refs/mortar-backup/<host>/`) from `refs.remote`. The caller holds lock `land` and
/// has run oplog recover (S-16). Only backup refs that map back under the four backed-up prefixes count. A local
/// ref already at the backup's id is Same; one at another id is Differs and is not moved. Each ref absent locally
/// is fetched to the same backup name here; one gone from the remote by then is Vanished and nothing is created
/// for it. The fetched ones are created, each at the id its fetched copy holds, by one op through apply (kind
/// `land-restore-backup`): Created, with that id. Nothing to create: no op. Every entry comes back in the remote's
/// order. A host that is empty or holds '/' is BadHost before any git run; a git failure is Git; an op refused or
/// repeated is OpFailed or OpRepeated, and then nothing was created.
pub fn restore_backup(git: &Git, refs: &LandRefs, host: &str, actor: &str) -> Result<Vec<Restored>, BackupError> {
    let prefix = backup_prefix(host)?;
    let remote = git.ls_remote(&refs.remote, &[]).map_err(BackupError::Git)?;
    let mut created: Vec<(String, Oid)> = Vec::new();
    let mut restored = Vec::new();
    for (backup, entry) in restore_plan(&prefix, &remote, &list_backed_up_refs(git)?) {
        restored.push(match entry {
            Restored::Created { refname, .. } => fetched(git, &refs.remote, &backup, refname)?,
            other => other,
        });
        if let Some(Restored::Created { refname, oid }) = restored.last() {
            created.push((refname.clone(), oid.clone()));
        }
    }
    if !created.is_empty() {
        let common = git.common_dir().map_err(BackupError::Git)?;
        restore_applied(apply(git, &common, &restore_op(host, &created, new_op_id(), actor, unix_now())))?;
    }
    Ok(restored)
}

/// The git runs for one local ref `refname` to be created: fetches `backup` from `remote` to the same name here,
/// then, only when it was fetched, reads what the copy holds; restored_after_fetch decides the answer.
fn fetched(git: &Git, remote: &str, backup: &str, refname: String) -> Result<Restored, BackupError> {
    let outcome = git.fetch_ref(remote, backup, backup).map_err(BackupError::Git)?;
    let copy = match outcome {
        FetchRefOutcome::Fetched => git.read_ref(backup).map_err(BackupError::Git)?,
        FetchRefOutcome::MissingSource => None,
    };
    Ok(restored_after_fetch(refname, outcome, copy))
}
