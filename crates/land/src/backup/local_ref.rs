//! local_ref: the local ref a backup ref restores to, the inverse of backup_ref. local_ref_tests cover it,
//! BACKUP_PREFIXES with it.

use crate::backup::BACKUP_PREFIXES;

/// `<prefix><rest>` -> `refs/<rest>`. None when `backup` is not under `prefix`, or when `refs/<rest>` is not
/// strictly inside one of BACKUP_PREFIXES: restore never creates any other local ref, such as `refs/heads/main`.
pub(crate) fn local_ref(prefix: &str, backup: &str) -> Option<String> {
    let local = format!("refs/{}", backup.strip_prefix(prefix)?);
    BACKUP_PREFIXES
        .iter()
        .any(|p| local.strip_prefix(p).is_some_and(|rest| !rest.is_empty()))
        .then_some(local)
}
