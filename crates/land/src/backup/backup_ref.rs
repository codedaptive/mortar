//! backup_ref: the backup ref on the remote that mirrors one local ref (SPEC-land 08). backup_ref_tests cover it.

/// `refs/<rest>` -> `<prefix><rest>`, with `prefix` from backup_prefix (ending in '/').
/// Callers pass only full local names under `refs/` (the ones listed under BACKUP_PREFIXES); a name without
/// `refs/` is appended whole.
pub(crate) fn backup_ref(prefix: &str, local: &str) -> String {
    format!("{prefix}{}", local.strip_prefix("refs/").unwrap_or(local))
}
