//! Decides whether step 5 may write to the moot copy it was given.

use super::Registered;
use std::path::Path;

/// Why the drive must not write to `copy` (canonical, with its canonical git common dir `copy_common`), or None
/// when it may. With no registration read it fails closed. A copy that is a registered path is refused first, over
/// the whole list; then one whose common dir is a registered path, that path's `.git`, or its recorded common dir
/// (a worktree of that repository). The first match in slice order names the repository.
pub fn copy_refusal(copy: &Path, copy_common: &Path, registered: &[Registered]) -> Option<String> {
    if registered.is_empty() {
        return Some(format!("no registered team repository was read, so the drive cannot tell {} is not one", copy.display()));
    }
    if let Some(r) = registered.iter().find(|r| copy == r.path) {
        return Some(format!("{} is the registered team repository {}", copy.display(), r.path.display()));
    }
    registered
        .iter()
        .find(|r| copy_common == r.path || copy_common == r.path.join(".git") || Some(copy_common) == r.common.as_deref())
        .map(|r| {
            format!(
                "{} is a worktree of the registered team repository {} (git common dir {})",
                copy.display(),
                r.path.display(),
                copy_common.display()
            )
        })
}
