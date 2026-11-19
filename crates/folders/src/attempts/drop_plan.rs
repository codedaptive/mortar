//! What drop_attempts removes and deletes, or why it refuses, before anything is touched.

use std::path::PathBuf;

use crate::make::FolderError;

use super::{unsaved_attempts, Attempt, Dropped};

/// Plans dropping every attempt in `attempts`. Refuses OrphanAttempt naming, in k order, the folder of every orphan
/// in `orphans` (orphan_attempts: (k, folder) for a folder whose attempt branch is gone), as remake_orphans must
/// re-make them first; then AttemptsUnsaved naming every attempt not listed in `saved` as (its k, true), in k order,
/// by its folder or, when it has none, by its full branch name (unsaved_attempts). Otherwise: every attempt's folder
/// in k order, skipping attempts with no folder (a repeat after a kill, given the same `saved` list); every
/// attempt's (branch, tip) in k order. No attempts plan an empty Dropped.
pub fn drop_plan(attempts: &[Attempt], orphans: &[(u32, PathBuf)], saved: &[(u32, bool)]) -> Result<Dropped, FolderError> {
    if !orphans.is_empty() {
        let mut sorted_orphans: Vec<&(u32, PathBuf)> = orphans.iter().collect();
        sorted_orphans.sort_by_key(|(k, _)| *k);
        return Err(FolderError::OrphanAttempt {
            paths: sorted_orphans.into_iter().map(|(_, folder)| folder.clone()).collect(),
        });
    }
    let unsaved = unsaved_attempts(attempts, saved, None);
    if !unsaved.is_empty() {
        return Err(FolderError::AttemptsUnsaved { paths: unsaved });
    }
    let mut sorted: Vec<&Attempt> = attempts.iter().collect();
    sorted.sort_by_key(|a| a.k);
    Ok(Dropped {
        remove: sorted.iter().filter_map(|a| a.folder.clone()).collect(),
        delete: sorted.iter().map(|a| (a.branch.clone(), a.tip.clone())).collect(),
    })
}
