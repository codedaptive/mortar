//! folder_setup_for: the repository, path, floor, sweep figure and actor every make call of mr folder gets.

use std::collections::BTreeMap;
use std::path::PathBuf;

use folders::SweepEstimate;
use gitio::Git;

use super::folder_setup_for;
use crate::config::Config;

#[test]
fn the_setup_carries_the_config_floor_the_actor_and_an_unmeasured_sweep() {
    let git = Git::new("/w/repo");
    let config = Config {
        origin_branch: "main".to_string(),
        integration_branch: "integration".to_string(),
        remote: "origin".to_string(),
        zones: BTreeMap::new(),
        root_files: Vec::new(),
        build_dirs: Vec::new(),
        cap_bytes: 1,
        floor_bytes: 4096,
        formatter: None,
    };
    let setup = folder_setup_for(&git, PathBuf::from("/w/fix"), &config, "harness");
    assert!(std::ptr::eq(setup.git, &git));
    assert_eq!(setup.path, PathBuf::from("/w/fix"));
    assert_eq!(setup.floor_bytes, 4096);
    assert_eq!(setup.sweep, SweepEstimate { bytes: 0, reason: Some("not measured by mr folder".to_string()) });
    assert_eq!(setup.actor, "harness");
}
