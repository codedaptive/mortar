//! Clears the build dirs of idle folders.
//! No unit test: it reads and deletes files (S-5); idle_folders_tests covers the choice. Nagatha's integration
//! group is to cover, in a throwaway repository: a build dir in a folder idle that long is deleted and its bytes
//! reported, while a folder touched since keeps its build dir; nothing else in the idle folder is deleted; the main
//! checkout's target/ is never touched; a build dir that is a symlink is left alone; a folder counted in disk's
//! sweep figure has its build dir cleared by sweep_build at the same idle_days.

use gitio::Git;
use oplog::unix_now;

use super::{idle_folders, FolderAge, SweepBuildOutcome, DEFAULT_BUILD_DIR};
use crate::disk::{measure_folder, mortar_folders};
use crate::make::FolderError;

/// Deletes DEFAULT_BUILD_DIR in each Mortar folder untouched for at least `idle_days` days: its newest mtime, by
/// measure_folder (the measure disk's sweep figure uses), is that old (idle_folders). Only a build dir that is a
/// directory, not a symlink, is deleted, and its bytes as measure_folder measured them are counted as freed;
/// nothing else in the folder is touched. The main checkout has no folder record, so it is never swept. An io
/// failure is Io naming the path. Takes no lock (S-16).
pub fn sweep_build(git: &Git, idle_days: u64) -> Result<SweepBuildOutcome, FolderError> {
    let mut measured = Vec::new();
    for (path, _) in mortar_folders(git)? {
        let measure = measure_folder(&path)?;
        measured.push((path, measure));
    }
    let ages: Vec<FolderAge> =
        measured.iter().map(|(path, m)| FolderAge { path: path.clone(), newest: m.newest }).collect();
    let idle = idle_folders(&ages, unix_now(), idle_days);
    let mut outcome = SweepBuildOutcome { removed: Vec::new(), freed_bytes: 0 };
    for (folder, measure) in measured.iter().filter(|(path, _)| idle.contains(path)) {
        let build = folder.join(DEFAULT_BUILD_DIR);
        let is_dir = match std::fs::symlink_metadata(&build) {
            Ok(meta) => meta.is_dir(),
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => false,
            Err(source) => return Err(FolderError::Io { path: build, source }),
        };
        if !is_dir {
            continue;
        }
        std::fs::remove_dir_all(&build).map_err(|source| FolderError::Io { path: build.clone(), source })?;
        outcome.freed_bytes = outcome.freed_bytes.saturating_add(measure.build);
        outcome.removed.push(build);
    }
    Ok(outcome)
}
