//! The branches and remote the guard is to protect, as the Mortar config names them.

use land::LandRefs;

use crate::config::Config;

/// The config's origin branch, integration branch and remote, as land's LandRefs.
pub fn guard_refs_for(config: &Config) -> LandRefs {
    LandRefs {
        origin_branch: config.origin_branch.clone(),
        integration_branch: config.integration_branch.clone(),
        remote: config.remote.clone(),
    }
}
