//! real_index_git: the Git value save sets the folder's index with, so it reaches the folder's own index (S-33).

use gitio::Git;

/// `git` with every env entry whose key is exactly GIT_INDEX_FILE removed; program, repo and the other entries,
/// in order, are kept. gitio applies Git.env after its own scrub of GIT_INDEX_FILE (git_set_index.rs), so a
/// caller's GIT_INDEX_FILE left on the value would point set_index at that index instead of the folder's own.
pub(crate) fn real_index_git(git: &Git) -> Git {
    Git {
        env: git.env.iter().filter(|(key, _)| key != "GIT_INDEX_FILE").cloned().collect(),
        ..git.clone()
    }
}
