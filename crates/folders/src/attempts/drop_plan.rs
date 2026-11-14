//! What drop_attempts removes and deletes, or why it refuses, before anything is touched (S-82).
//! Its tests are zone integration tests in tests/folders/attempt_plans.rs: it uses unsaved_attempts for real (#1264).

use std::path::PathBuf;

use crate::make::FolderError;

use super::{unsaved_attempts, Attempt, Dropped};

/// Plans dropping every attempt in `attempts` and every attempt folder in `orphans` (orphan_attempts: (k, folder)
/// for a folder whose attempt branch is gone). Refuses AttemptsUnsaved naming every attempt not listed in `saved` as
/// (its k, true), in k order, by its folder or, when it has none, by its full branch name (unsaved_attempts); then
/// every such orphan, in the order given, by its folder. Otherwise: every attempt's folder in k order, skipping
/// attempts with no folder (a repeat after a kill, given the same `saved` list), then every orphan's folder in k
/// order; every attempt's (branch, tip) in k order, and nothing for orphans, which have no branch. No attempts and
/// no orphans plan an empty Dropped.
pub fn drop_plan(attempts: &[Attempt], orphans: &[(u32, PathBuf)], saved: &[(u32, bool)]) -> Result<Dropped, FolderError> {
    let mut unsaved = unsaved_attempts(attempts, saved, None);
    unsaved.extend(orphans.iter().filter(|(k, _)| !saved.contains(&(*k, true))).map(|(_, folder)| folder.clone()));
    if !unsaved.is_empty() {
        return Err(FolderError::AttemptsUnsaved { paths: unsaved });
    }
    let mut sorted: Vec<&Attempt> = attempts.iter().collect();
    sorted.sort_by_key(|a| a.k);
    let mut sorted_orphans: Vec<&(u32, PathBuf)> = orphans.iter().collect();
    sorted_orphans.sort_by_key(|(k, _)| *k);
    Ok(Dropped {
        remove: sorted
            .iter()
            .filter_map(|a| a.folder.clone())
            .chain(sorted_orphans.iter().map(|(_, folder)| folder.clone()))
            .collect(),
        delete: sorted.iter().map(|a| (a.branch.clone(), a.tip.clone())).collect(),
    })
}
