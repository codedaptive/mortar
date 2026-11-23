//! zone_folder_of: the path of a zone's zone folder, found from the worktrees' folder records; `mr folder request`
//! shares the build dirs from it. It holds the rule that picks the zone folder: the first worktree in git's order,
//! bare and prunable ones skipped, whose folder record is a Zone record of the zone; a worktree whose git dir or
//! record cannot be read is passed over.
//! No sibling test: the rule reads git's worktree list and the records on disk, so it is tested for real. The
//! integration tests in crates/cli/tests/folder_runs/run_folder_request_tests.rs cover it through the binary: a zone
//! folder found and its build dirs shared (request_folder_build_output_shared, request_folder_build_output_absent,
//! request_folder_build_output_kept_on_repeat, request_folder_build_output_refused_without_failing), and a zone with
//! no zone folder (request_folder_holds_only_its_paths_and_the_root_files).

use std::path::{Path, PathBuf};

use folders::make::read_folder_record;
use folders::FolderKind;
use gitio::Git;

/// The first worktree in git's order, bare and prunable ones skipped, whose folder record is a Zone record of
/// `zone`. A worktree whose git dir or record cannot be read is passed over, so one bad record never fails the
/// lookup. Err: `git could not list the repository's worktrees`, or `found no zone folder of zone {zone}` (no
/// worktree carries a readable Zone record of the zone).
pub fn zone_folder_of(git: &Git, zone: &str) -> Result<PathBuf, String> {
    git.worktree_list()
        .map_err(|_| "git could not list the repository's worktrees".to_string())?
        .into_iter()
        .filter(|worktree| !worktree.bare && !worktree.prunable)
        .find(|worktree| is_zone_folder_of(git, &worktree.path, zone))
        .map(|worktree| worktree.path)
        .ok_or_else(|| format!("found no zone folder of zone {zone}"))
}

/// Whether the worktree at `path` carries a readable Zone record of `zone`.
fn is_zone_folder_of(git: &Git, path: &Path, zone: &str) -> bool {
    let Ok(git_dir) = git.git_dir(path) else {
        return false;
    };
    matches!(read_folder_record(&git_dir), Ok(Some(record)) if record.kind == FolderKind::Zone && record.zone == zone)
}
