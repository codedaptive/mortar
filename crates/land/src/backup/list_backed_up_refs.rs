//! list_backed_up_refs: the local refs backup mirrors and restore_backup compares against.
//! No unit test: it runs git; the backup integration tests (backup_zit) cover it.

use crate::backup::{BackupError, BACKUP_PREFIXES};
use gitio::{Git, Oid};

/// Every local ref under each of BACKUP_PREFIXES as (id, full refname): the prefixes in their order, each
/// prefix's refs in git's order.
pub(crate) fn list_backed_up_refs(git: &Git) -> Result<Vec<(Oid, String)>, BackupError> {
    let mut refs = Vec::new();
    for prefix in BACKUP_PREFIXES {
        refs.extend(git.list_refs(prefix).map_err(BackupError::Git)?);
    }
    Ok(refs)
}
