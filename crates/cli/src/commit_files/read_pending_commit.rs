//! read_pending_commit: the commit a stopped `mr commit-files` run in a zone folder staged its named paths on, read
//! from the folder's pending marker when that marker accounts for the folder's index.
//! No sibling test: it reads a file and runs git; parse_pending_commit_tests covers the marker's parsing. The
//! integration tests in crates/cli/tests/folder_runs/run_commit_files_tests.rs cover it through the binary:
//! commit_files_zone_repeat_after_a_stopped_run_commits_once (a marker at the tip lets the repeat commit once) and
//! commit_files_zone_lagging_with_a_staged_named_path_is_refused (no marker, so the staged path is refused).

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use gitio::{Git, Oid};

use super::parse_pending_commit;

/// The marker's tip when the marker at `marker` counts: its paths are the same set as `paths`, in any order, and the
/// tip's tree with `paths` taken from `index_tree` (gitio's tree_with_paths_typed) is `index_tree`, so the index
/// differs from that tip only at those paths. None when the marker is missing or unparsable, names other paths,
/// does not account for the index, or git fails or refuses the paths while it is checked.
pub fn read_pending_commit(git: &Git, marker: &Path, index_tree: &Oid, paths: &[PathBuf]) -> Option<Oid> {
    let pending = parse_pending_commit(&fs::read_to_string(marker).ok()?)?;
    let set = |paths: &[PathBuf]| paths.iter().cloned().collect::<BTreeSet<PathBuf>>();
    if set(&pending.paths) != set(paths) {
        return None;
    }
    let tip_tree = git.commit_info(&pending.tip).ok()?.tree;
    let overlay = git.tree_with_paths_typed(&tip_tree, index_tree, paths).ok()?;
    (overlay == *index_tree).then_some(pending.tip)
}
