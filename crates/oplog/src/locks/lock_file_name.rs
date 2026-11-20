//! The file name of a lock: `land.lock`, `folders.lock`, or `join-<zone>.lock` for a zone name
//! safe in a file name.

use super::{LockError, LockName};

/// Longest zone accepted, in bytes: keeps `join-<zone>.lock` well under common file name limits (255).
const MAX_ZONE_BYTES: usize = 100;

/// The lock's file name. A zone that could escape the locks folder or hide the file is refused.
pub(crate) fn lock_file_name(name: &LockName) -> Result<String, LockError> {
    match name {
        LockName::Land => Ok("land.lock".to_string()),
        LockName::Folders => Ok("folders.lock".to_string()),
        LockName::Join(zone) if zone_is_valid(zone) => Ok(format!("join-{zone}.lock")),
        LockName::Join(zone) => Err(LockError::BadName(zone.clone())),
    }
}

fn zone_is_valid(zone: &str) -> bool {
    !zone.is_empty()
        && zone.len() <= MAX_ZONE_BYTES
        && !zone.starts_with('.')
        && zone
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}
