//! The refs Mortar keeps forward-only, from the config.

use oplog::ProtectedRef;

use super::Config;

/// Only the origin branch is protected; the integration branch is guarded but may go back to origin.
pub fn protected_refs(config: &Config) -> Vec<ProtectedRef> {
    vec![ProtectedRef {
        refname: format!("refs/heads/{}", config.origin_branch),
        by: "mortar config origin_branch".to_string(),
    }]
}
