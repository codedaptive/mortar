//! Shares a parent folder's build dirs with a newly made folder.
//! No unit test: it reads and writes the filesystem; tests/folders/build_output.rs covers it.

use std::fs;
use std::path::Path;

use super::{ancestor_refusal, build_dir_outcome, clonefile_dir, symlinked_ancestor, BuildDirClone};

/// Clones each of `build_dirs` from `parent` into `folder` with clonefile(2), in order, and answers one
/// outcome per dir. `parent` is the folder whose build output is shared (for a request folder, its zone
/// folder); `folder` is the newly made one: call this after make_request_folder or make_zone_folder returns.
/// `build_dirs` are the caller's, relative to both folders (a caller with no list passes DEFAULT_BUILD_DIR). A dir
/// with a symlinked ancestor in either folder is Refused naming it, before anything is cloned or created. The probe
/// and the clone are separate steps, so a symlink a concurrent writer makes in between is not seen. A dir already
/// in `folder` is Kept, never overwritten or merged, so a repeat make is safe. A refused dir has nothing copied:
/// there is never a byte copy.
pub fn clone_build_dirs(parent: &Path, folder: &Path, build_dirs: &[String]) -> Vec<BuildDirClone> {
    build_dirs
        .iter()
        .map(|dir| {
            if let Some(refused) = ancestor_refusal(
                dir,
                symlinked_ancestor(parent, dir).as_deref(),
                symlinked_ancestor(folder, dir).as_deref(),
            ) {
                return refused;
            }
            let source = parent.join(dir);
            let target = folder.join(dir);
            let found = fs::symlink_metadata(&source).ok();
            let parent_is_symlink = found.as_ref().is_some_and(|meta| meta.file_type().is_symlink());
            let in_folder = fs::symlink_metadata(&target).is_ok();
            build_dir_outcome(dir, found.is_some(), parent_is_symlink, in_folder, || {
                if let Some(holder) = target.parent() {
                    fs::create_dir_all(holder)
                        .map_err(|e| format!("could not make {}: {e}; nothing copied", holder.display()))?;
                }
                clonefile_dir(&source, &target)
            })
        })
        .collect()
}
