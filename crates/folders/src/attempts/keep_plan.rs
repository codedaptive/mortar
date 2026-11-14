//! What keep(k) removes and deletes, or why it refuses, before anything is touched (S-69).

use std::path::PathBuf;

use crate::make::FolderError;

use super::{Attempt, Kept};

/// Plans keeping attempt `k` of `attempts`. Refuses AttemptMissing (naming `kept_branch`) when no attempt is numbered k,
/// then AttemptsUnsaved naming, in k order, every other attempt not listed in `saved` as (its k, true): those would
/// be dropped. A dropped attempt is named by its folder, or by its full branch name as a path when it has no folder,
/// so FolderError needs no second variant. Attempt k needs no saved entry: its commits stay on the request branch,
/// and git's no-force remove refuses its uncommitted files. Otherwise: the kept tip; the dropped attempts' folders
/// in k order then attempt k's last, skipping attempts with no folder (a repeat after a kill, given the same `saved`
/// list); every attempt's (branch, tip), attempt k's included, in k order.
pub fn keep_plan(kept_branch: &str, k: u32, attempts: &[Attempt], saved: &[(u32, bool)]) -> Result<Kept, FolderError> {
    let mut sorted: Vec<&Attempt> = attempts.iter().collect();
    sorted.sort_by_key(|a| a.k);
    let kept = sorted
        .iter()
        .find(|a| a.k == k)
        .ok_or_else(|| FolderError::AttemptMissing {
            branch: kept_branch.to_string(),
        })?;
    let unsaved: Vec<PathBuf> = sorted
        .iter()
        .filter(|a| a.k != k && !saved.contains(&(a.k, true)))
        .map(|a| a.folder.clone().unwrap_or_else(|| PathBuf::from(&a.branch)))
        .collect();
    if !unsaved.is_empty() {
        return Err(FolderError::AttemptsUnsaved { paths: unsaved });
    }
    Ok(Kept {
        tip: kept.tip.clone(),
        remove: sorted
            .iter()
            .filter(|a| a.k != k)
            .filter_map(|a| a.folder.clone())
            .chain(kept.folder.clone())
            .collect(),
        delete: sorted.iter().map(|a| (a.branch.clone(), a.tip.clone())).collect(),
    })
}
