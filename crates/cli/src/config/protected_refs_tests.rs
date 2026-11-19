//! protected_refs: only the origin branch, by the config's origin_branch.

use std::collections::BTreeMap;

use oplog::ProtectedRef;

use super::{protected_refs, Config};

#[test]
fn only_the_origin_branch_is_protected() {
    let config = Config {
        origin_branch: "release/1".to_string(),
        integration_branch: "integration".to_string(),
        remote: "origin".to_string(),
        zones: BTreeMap::new(),
        root_files: Vec::new(),
        build_dirs: Vec::new(),
        cap_bytes: 1,
        floor_bytes: 1,
        formatter: None,
    };
    assert_eq!(
        protected_refs(&config),
        vec![ProtectedRef { refname: "refs/heads/release/1".to_string(), by: "mortar config origin_branch".to_string() }]
    );
}
