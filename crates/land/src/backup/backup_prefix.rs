//! backup_prefix: the remote namespace one host's backup lives under. backup_prefix_tests cover it.

use crate::backup::BackupError;

/// `refs/mortar-backup/<host>/`. A host that is empty or holds '/' would name no namespace or another host's,
/// so it is refused as `BadHost` before any git run.
pub(crate) fn backup_prefix(host: &str) -> Result<String, BackupError> {
    if host.is_empty() || host.contains('/') {
        return Err(BackupError::BadHost(host.to_string()));
    }
    Ok(format!("refs/mortar-backup/{host}/"))
}
