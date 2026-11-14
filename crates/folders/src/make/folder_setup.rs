//! What making a folder needs besides its names and paths: the repository, where, the disk floor, and actors.
//! A bare type; the make functions read it.

use std::path::PathBuf;

use gitio::Git;
use oplog::PidProbe;

use crate::plan::{FreeDisk, SweepEstimate};

/// The setup for one make_zone_folder or make_request_folder call. `path` is the folder to make;
/// `floor_bytes` is the free-disk floor, DEFAULT_FLOOR_BYTES unless the caller chooses another; `sweep` is
/// what a sweep would free, reported when free disk is under the floor; `disk` answers free disk; `pids`
/// answers whether a lock holder is alive; `actor` names who makes the folder in the op log.
pub struct FolderSetup<'a> {
    pub git: &'a Git,
    pub path: PathBuf,
    pub floor_bytes: u64,
    pub sweep: SweepEstimate,
    pub disk: &'a dyn FreeDisk,
    pub pids: &'a dyn PidProbe,
    pub actor: String,
}
