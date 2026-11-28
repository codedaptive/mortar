//! commit_files_with_hooks: commit_files with the repository's pre-commit and commit-msg hooks run on the commit, as
//! `git commit` runs them. It takes no lock: the caller holds `land` and has run recover, as for commit_files.
//! No unit test: it runs git and the hooks. tests/commit_files_zit/hooks.rs covers it in a throwaway repository:
//! pre_commit_refusal_leaves_landing_and_index_unchanged, commit_msg_refusal_leaves_landing_and_index_unchanged,
//! commit_msg_rewrite_becomes_the_committed_message and same_content_skips_refusing_pre_commit.

use crate::landing::{commit_files_steps, CommitFilesAnswer, HookedCommitFilesError};
use crate::squash::LandRefs;
use gitio::Git;
use std::path::PathBuf;

/// Commits `paths` of the integration folder onto integration as commit_files does, with the pre-commit and
/// commit-msg hooks run as `git commit` would run them, through snapshot's commit_paths_with_hooks. Same runs no
/// hooks and moves nothing: no commit, no ref, no op, no landing-state write. A refusing hook is Refused with no
/// commit, no landing-state write, no op, integration unmoved and the folder's index untouched. The committed
/// message is the one commit-msg leaves. Every other refusal is commit_files' refusal inside CommitFiles: snapshot's
/// Paths(e) as CommitFiles(CommitFilesError::Paths(e)), the mapping hooked_commit_files_error_from_hooks makes.
pub fn commit_files_with_hooks(
    git: &Git,
    refs: &LandRefs,
    paths: &[PathBuf],
    message: &str,
    actor: &str,
) -> Result<CommitFilesAnswer, HookedCommitFilesError> {
    commit_files_steps(git, refs, actor, |tip| {
        snapshot::commit_paths_with_hooks(git, &git.repo, tip, paths, message).map_err(HookedCommitFilesError::from)
    })
}
