//! would_merge: whether a zone's squash merges into integration, by a trial merge.
//! No unit test: it runs git (S-5); would_merge_from_tests covers its answer. tests/squash_zit.rs covers it, in a
//! throwaway repository: would_merge yes and no.

use crate::squash::{check_attr_source, read_tip, would_merge_from, LandRefs, Squash, SquashError, WouldMerge};
use gitio::Git;

/// Merges `squash.commit` into the tip of `refs/heads/<integration_branch>` with that tip as the attribute source
/// (checked to exist first): Yes when clean, No with the conflicted files. No ref moves and no commit is
/// written, but git's merge-tree writes the merged tree objects, which nothing keeps (not read only).
pub fn would_merge(git: &Git, refs: &LandRefs, squash: &Squash) -> Result<WouldMerge, SquashError> {
    let integration = read_tip(git, &format!("refs/heads/{}", refs.integration_branch))?;
    check_attr_source(git, &integration)?;
    let outcome = git.merge_tree(&integration, None, &integration, &squash.commit).map_err(SquashError::Git)?;
    Ok(would_merge_from(outcome))
}
