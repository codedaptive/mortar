//! remove's attempts check: a request folder is not removed while its attempts remain (S-82). No unit test: it
//! runs git to list the try refs and the worktrees (S-5). tests/folders/attempt_plans.rs covers what counts as an
//! attempt left. Nagatha's integration group is to cover, in a throwaway repository: a request folder with an attempt
//! branch left is AttemptsRemain, with nothing changed; one with an attempt folder left (its branch present)
//! likewise; after keep or drop_attempts, remove goes ahead.

use std::path::Path;

use gitio::Git;

use crate::attempts::attempts_remaining;
use crate::make::FolderError;

/// Ok when nothing of `request`'s attempts in `zone` is left: no attempt branch and no attempt folder, as
/// attempts_remaining counts them from the refs under `refs/heads/try/<zone>/<request>/` and the worktree list.
/// Otherwise AttemptsRemain for `folder`, naming the branches by full name and the folders by path as display text;
/// the next step is keep or drop_attempts. A git error listing refs or worktrees passes through as Git.
pub(crate) fn attempts_refusal(git: &Git, folder: &Path, zone: &str, request: &str) -> Result<(), FolderError> {
    let try_refs = git
        .list_refs(&format!("refs/heads/try/{zone}/{request}/"))
        .map_err(FolderError::Git)?;
    let worktrees = git.worktree_list().map_err(FolderError::Git)?;
    let attempts = attempts_remaining(zone, request, &try_refs, &worktrees);
    if attempts.is_empty() {
        return Ok(());
    }
    Err(FolderError::AttemptsRemain {
        path: folder.to_path_buf(),
        attempts,
    })
}
