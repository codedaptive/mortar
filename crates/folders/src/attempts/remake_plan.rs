//! What remake_orphans re-makes, or why it refuses, before anything is touched (S-96 (2), S-98).

use std::path::PathBuf;

use crate::make::FolderError;

use super::{Remake, Snap};

/// Plans re-making every orphan in `orphans` ((k, folder) for a folder whose attempt branch is gone) from its
/// snapshot in `snaps` ((k, the commit its trysnap ref names, None when there is no such ref)). Refuses
/// OrphanWithoutSnapshot naming, in k order, the folder of every orphan with no snapshot (no entry, or None) or whose
/// snapshot has no first parent; nothing is planned then. Otherwise one Remake per orphan, in k order, at its
/// snapshot's tree and first parent. No orphans plan nothing.
pub fn remake_plan(orphans: &[(u32, PathBuf)], snaps: &[(u32, Option<Snap>)]) -> Result<Vec<Remake>, FolderError> {
    let mut sorted: Vec<&(u32, PathBuf)> = orphans.iter().collect();
    sorted.sort_by_key(|(k, _)| *k);
    let snap_of = |k: u32| snaps.iter().find(|(s, _)| *s == k).and_then(|(_, snap)| snap.as_ref());
    let mut remakes = Vec::new();
    let mut missing = Vec::new();
    for (k, folder) in sorted {
        match snap_of(*k).and_then(|snap| snap.first_parent.clone().map(|parent| (snap.tree.clone(), parent))) {
            Some((tree, parent)) => remakes.push(Remake { k: *k, folder: folder.clone(), tree, parent }),
            None => missing.push(folder.clone()),
        }
    }
    if !missing.is_empty() {
        return Err(FolderError::OrphanWithoutSnapshot { paths: missing });
    }
    Ok(remakes)
}
