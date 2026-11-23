//! LandRefs: the branches land reads. A declaration only; tests/squash_zit.rs exercises it.

/// Branch names, not full refs: each tip is read from `refs/heads/<branch>`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LandRefs {
    /// The branch a squash is built on and its change ID is checked against.
    pub origin_branch: String,
    /// The branch would_merge and would_merge_with merge a squash into.
    pub integration_branch: String,
    /// The remote land publishes to.
    pub remote: String,
}
