//! make_landing_target: the commit a landing moves integration to, writing the merge commit when one is needed.
//! No unit test: it runs git; is_fast_forward_tests covers its decisions. Covered by the
//! landing_zit tests a_squash_on_origin_fast_forwards_integration_to_the_squash,
//! a_squash_behind_a_moved_origin_lands_as_a_merge_commit (parents, message, tree, the squash commit unchanged) and
//! a_conflicting_squash_moves_nothing_and_writes_no_state.

use crate::landing::{is_fast_forward, landing_message, LandError, LandingTarget};
use crate::squash::{check_attr_source, Squash};
use gitio::{Git, MergeTreeOutcome, Oid};

/// A squash whose only parent is `integration` is its own target, and nothing is written. Otherwise the squash is
/// merged into `integration`, with `integration` as the attribute source (checked to exist first): clean, a commit
/// of the merged tree with parents `[integration, squash]` and the landing message naming the change ID and `zone`;
/// conflicted, the files, and no commit and no ref are written (git's merge-tree still writes tree objects). The
/// squash commit is never rewritten.
pub(crate) fn make_landing_target(
    git: &Git,
    integration: &Oid,
    squash: &Squash,
    parents: &[Oid],
    zone: &str,
) -> Result<LandingTarget, LandError> {
    if is_fast_forward(parents, integration) {
        return Ok(LandingTarget::Commit(squash.commit.clone()));
    }
    check_attr_source(git, integration).map_err(LandError::Squash)?;
    let tree = match git.merge_tree(integration, None, integration, &squash.commit).map_err(LandError::Git)? {
        MergeTreeOutcome::Clean(tree) => tree,
        MergeTreeOutcome::Conflicted { files, .. } => return Ok(LandingTarget::Conflicted(files)),
    };
    let message = landing_message(&squash.change_id, zone);
    let parents = [integration.clone(), squash.commit.clone()];
    let commit = git.commit_tree(&tree, &parents, &message, None, None).map_err(LandError::Git)?;
    Ok(LandingTarget::Commit(commit))
}
