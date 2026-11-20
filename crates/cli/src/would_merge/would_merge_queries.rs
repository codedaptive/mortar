//! The git read mr would-merge's answer replaces.

use gitio::{BaselineQuery, Oid};

/// `git log -p refs/heads/<integration_branch>..<commit>` over the whole tree (no paths): the squash's commits not
/// yet on the integration branch, which land's would_merge merges into.
pub fn would_merge_queries(integration_branch: &str, commit: &Oid) -> Vec<BaselineQuery> {
    vec![BaselineQuery::LogPatches { range: format!("refs/heads/{integration_branch}..{commit}"), paths: vec![] }]
}
