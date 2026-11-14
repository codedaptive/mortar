//! LandRefs: the branches land reads. A declaration only (S-7); Nagatha's squash and would-merge integration tests of
//! SPEC-land 01-squash.md §Tests are to exercise it (they do not exist yet).

/// Branch names, not full refs: each tip is read from `refs/heads/<branch>`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LandRefs {
    /// The branch a squash is built on and its change ID is checked against.
    pub origin_branch: String,
    /// The branch would_merge merges a squash into.
    pub integration_branch: String,
    /// The remote land publishes to.
    pub remote: String,
}
