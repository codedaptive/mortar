//! CommitPathsOutcome: what commit_paths left behind. A declaration only; the integration tests in
//! tests/snapshot/commit_paths_cases.rs read both variants (commit_paths.rs's header lists their cases).

use gitio::Oid;

/// Committed: the commit written, whose one parent is the caller's and whose tree is `tree`; no ref names it yet.
/// Same: the named paths already stood in the parent's tree as in the folder, so no commit was written and the
/// folder's index was left as it was.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommitPathsOutcome {
    Committed { commit: Oid, tree: Oid },
    Same,
}
