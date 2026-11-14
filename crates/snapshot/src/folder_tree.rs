//! folder_tree: a folder's HEAD commit, the tree of its files as a snapshot saves them, and the files the size cap
//! left out.
//! No unit test: it only runs git and reads file sizes, and holds no decision of its own (S-5);
//! snapshot_excludes_tests covers the excludes it passes, and cap_candidates, cap_refusals, capped_excludes, cap_mismatch, empty_tree and
//! candidate_size are its decisions and have their tests. Nagatha's integration group should cover, in a
//! throwaway repository: build output directories (top-level and nested target/, node_modules/, .build/),
//! .gitignored files and a caller exclude are left out of the tree; a sparse folder's tree keeps its out-of-cone
//! files; the folder's index and files are unchanged afterwards; a folder with an unborn HEAD gives UnbornHead;
//! a new 6 MB file is refused and listed, and is absent from the tree; a 6 MB file already in HEAD and changed is
//! kept with its new contents; a new file of exactly the cap is kept; a refused name holding glob characters
//! leaves out only itself; a HEAD directory replaced by a new over-cap file of the same name: the file is refused
//! and the directory's files are gone from the tree; with GIT_NOGLOB_PATHSPECS=1 a refused glob-named file gives
//! CapMismatch.

use crate::cap_candidates::cap_candidates;
use crate::cap_mismatch::cap_mismatch;
use crate::cap_refusals::cap_refusals;
use crate::capped_excludes::capped_excludes;
use crate::empty_tree::empty_tree;
use crate::file_size::file_size;
use crate::snapshot_excludes::snapshot_excludes;
use crate::{RefusedFile, SnapshotError};
use gitio::{Git, Oid};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// (HEAD commit, tree, refused) of `folder`. `git` must run in the folder (its `repo` is the folder's top), since
/// gitio's snapshot_tree saves the files of `git.repo`. The tree starts from HEAD so files outside a sparse cone
/// are kept, and leaves out the built-in excludes, `extra_excludes` and each file not in HEAD whose size in bytes
/// is over `size_cap`; those files are `refused`, in git's order. HEAD's paths are listed only when some changed
/// file is over the cap. Writes tree objects; moves no ref. A capped tree that does not differ from the full tree
/// by exactly the refused files gives CapMismatch.
pub(crate) fn folder_tree(
    git: &Git,
    folder: &Path,
    extra_excludes: &[String],
    size_cap: u64,
) -> Result<(Oid, Oid, Vec<RefusedFile>), SnapshotError> {
    let Some(head) = git.head(folder).map_err(SnapshotError::Git)?.1 else {
        return Err(SnapshotError::UnbornHead { folder: folder.to_path_buf() });
    };
    let full = git
        .snapshot_tree(&head, &snapshot_excludes(extra_excludes))
        .map_err(SnapshotError::Git)?;
    let sized = sized_changes(git, folder, &head, &full)?;
    let first = cap_refusals(&cap_candidates(&sized, None), size_cap);
    if first.is_empty() {
        return Ok((head, full, first));
    }
    let head_paths: HashSet<PathBuf> = git
        .changed_files(&empty_tree(&head).map_err(SnapshotError::Git)?, &head)
        .map_err(SnapshotError::Git)?
        .into_iter()
        .collect();
    let refused = cap_refusals(&cap_candidates(&sized, Some(&head_paths)), size_cap);
    if refused.is_empty() {
        return Ok((head, full, refused));
    }
    let capped = git
        .snapshot_tree(&head, &snapshot_excludes(&capped_excludes(extra_excludes, &refused)?))
        .map_err(SnapshotError::Git)?;
    let mismatch = cap_mismatch(&refused, &git.changed_files(&full, &capped).map_err(SnapshotError::Git)?);
    if !mismatch.is_empty() {
        return Err(SnapshotError::CapMismatch { paths: mismatch });
    }
    Ok((head, capped, refused))
}

/// Each path that differs between `head` and `full` with its size, skipping those file_size gives None for.
fn sized_changes(git: &Git, folder: &Path, head: &Oid, full: &Oid) -> Result<Vec<(PathBuf, u64)>, SnapshotError> {
    let mut sized = Vec::new();
    for path in git.changed_files(head, full).map_err(SnapshotError::Git)? {
        if let Some(size) = file_size(folder, &path)? {
            sized.push((path, size));
        }
    }
    Ok(sized)
}
