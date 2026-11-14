//! folder_tree: a folder's HEAD commit, the tree of its files as a snapshot saves them, and the files the size cap
//! left out.
//! No unit test: it only runs git and reads file sizes, and holds no decision of its own (S-5);
//! snapshot_excludes_tests covers the excludes it passes, and cap_candidates, cap_refusals, capped_excludes, cap_mismatch, empty_tree and
//! candidate_size are its decisions and have their tests.
//! Nagatha's integration tests cover it through snapshot, in a throwaway repository:
//! - build output directories (top-level and nested target/, node_modules/, .build/), .gitignored files and a caller
//!   exclude are left out of the tree, and similar names are kept:
//!   tests/snapshot/sparse_excludes.rs::ignored_build_outputs_and_caller_excludes_stay_out_but_similar_names_are_saved;
//! - a sparse folder's tree keeps its out-of-cone files:
//!   sparse_excludes.rs::sparse_snapshot_keeps_out_of_cone_tree_and_preserves_staged_index_and_all_files;
//! - the folder's index and files are unchanged afterwards: both sparse_excludes.rs tests;
//! - a new 6 MiB file is refused and listed, and is absent from the tree; a 6 MiB file already in HEAD and changed is
//!   kept with its new contents: tests/snapshot/caps.rs::new_six_mib_file_is_reported_but_large_changed_head_file_is_kept;
//! - a new file of exactly the cap is kept; a refused name holding glob characters leaves out only itself:
//!   caps.rs::caller_cap_is_inclusive_and_refused_glob_name_excludes_only_that_path;
//! - a file new against HEAD that grows over the cap, and then is renamed, is refused and absent from the tree:
//!   caps.rs::new_against_head_file_that_grows_or_is_renamed_over_cap_drops_from_next_snapshot;
//! - an unborn HEAD gives UnbornHead, through snapshot and is_saved, with nothing written:
//!   tests/snapshot/sizing_edges.rs::unborn_head_is_named_before_any_snapshot_commit_ref_or_op;
//! - a HEAD directory replaced by a new over-cap file of the same name: the file is refused and the directory's files
//!   are gone from the tree:
//!   sizing_edges.rs::over_cap_file_replacing_head_directory_is_refused_without_resurrecting_old_children;
//! - with GIT_NOGLOB_PATHSPECS=1 in the caller's Git env, a refused glob-named file gives CapMismatch naming it,
//!   through snapshot and is_saved:
//!   sizing_edges.rs::caller_noglob_environment_produces_exact_cap_mismatch_without_losing_previous_snapshot.

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
