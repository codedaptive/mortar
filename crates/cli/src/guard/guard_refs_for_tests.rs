//! guard_refs_for: each of the config's three names lands in its own LandRefs field.

use std::collections::BTreeMap;

use land::LandRefs;

use super::guard_refs_for;
use crate::config::Config;

#[test]
fn the_configs_branches_and_remote_each_in_their_field() {
    let config = Config {
        origin_branch: "release".to_string(),
        integration_branch: "develop".to_string(),
        remote: "upstream".to_string(),
        zones: BTreeMap::new(),
        root_files: Vec::new(),
        build_dirs: Vec::new(),
        cap_bytes: 1,
        floor_bytes: 0,
        formatter: None,
    };
    assert_eq!(
        guard_refs_for(&config),
        LandRefs {
            origin_branch: "release".to_string(),
            integration_branch: "develop".to_string(),
            remote: "upstream".to_string(),
        }
    );
}
