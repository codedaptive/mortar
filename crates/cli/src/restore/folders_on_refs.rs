//! folders_on_refs: the Mortar folders an `mr restore` would move, found by walking the repository's worktrees.
//! No unit test: it runs git and reads folder records; restore_refusal_tests and restore_report_tests
//! cover what is decided from what it finds, and run_restore_tests restore's usage refusals, before any lock.
//! The repo integration tests are to cover, with the mortar binary as a child process in a throwaway repository (tests running real git set GIT_CONFIG_GLOBAL=/dev/null and GIT_CONFIG_NOSYSTEM=1):
//! - a request folder on req/<z>/<r> is moved to the restored tip;
//! - a zone folder on zone/<z> is listed and not moved.

use folders::make::read_folder_record;
use gitio::Git;

use super::FolderOnRef;

/// Every worktree but bare and prunable ones whose branch (a full name; a detached worktree has none) is one of
/// `refs` is read for its folder record; a worktree with no record is not a Mortar folder and is skipped. Worktrees
/// on other refs are not read. In git's order. A git run or a record that cannot be read is Err with the reason.
pub fn folders_on_refs(git: &Git, refs: &[String]) -> Result<Vec<FolderOnRef>, String> {
    let mut found = Vec::new();
    for worktree in git.worktree_list().map_err(|error| error.to_string())? {
        if worktree.bare || worktree.prunable {
            continue;
        }
        let Some(refname) = worktree.branch.filter(|branch| refs.contains(branch)) else {
            continue;
        };
        let git_dir = git.git_dir(&worktree.path).map_err(|error| error.to_string())?;
        let record = read_folder_record(&git_dir).map_err(|error| {
            format!("the folder record of {} cannot be read: {error}", worktree.path.display())
        })?;
        if let Some(record) = record {
            found.push(FolderOnRef { path: worktree.path, refname, record });
        }
    }
    Ok(found)
}
