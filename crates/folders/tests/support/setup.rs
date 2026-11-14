use folders::{DfFreeDisk, FolderSetup, SweepEstimate};
use gitio::Git;
use oplog::KillPidProbe;
use std::path::Path;

pub fn setup<'a>(git: &'a Git, path: &Path) -> FolderSetup<'a> {
    FolderSetup {
        git, path: path.to_owned(), floor_bytes: 0,
        sweep: SweepEstimate { bytes: 1234, reason: Some("fixture sweep estimate".into()) },
        disk: &DfFreeDisk, pids: &KillPidProbe, actor: "folder-integration-caller".into(),
    }
}
