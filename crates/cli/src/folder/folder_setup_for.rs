//! What every folders make call of `mortar folder` is given besides its names and paths.

use std::path::PathBuf;

use folders::{DfFreeDisk, FolderSetup, SweepEstimate};
use gitio::Git;
use oplog::KillPidProbe;

use crate::config::Config;

/// The folder (or attempts' directory) `path`, the config's floor_bytes, free disk read by df (DfFreeDisk), lock
/// holders probed by kill (KillPidProbe), and `actor` for every op logged. The sweep figure is 0 bytes with the
/// reason "not measured by mortar folder": mortar folder does not measure what a sweep would free.
pub fn folder_setup_for<'a>(git: &'a Git, path: PathBuf, config: &Config, actor: &str) -> FolderSetup<'a> {
    FolderSetup {
        git,
        path,
        floor_bytes: config.floor_bytes,
        sweep: SweepEstimate { bytes: 0, reason: Some("not measured by mortar folder".to_string()) },
        disk: &DfFreeDisk,
        pids: &KillPidProbe,
        actor: actor.to_string(),
    }
}
