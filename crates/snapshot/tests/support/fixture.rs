use gitio::{Git, Oid};
use oplog::AcquiredLock;
use snapshot::SnapshotTarget;
use std::path::PathBuf;
use testkit::TestRepo;

/// Holds the caller's zone lock for the entire test, including recovery and snapshots.
pub struct Fixture {
    pub repo: TestRepo,
    pub git: Git,
    pub head: Oid,
    pub common: PathBuf,
    pub target: SnapshotTarget,
    pub refname: String,
    pub lock: AcquiredLock,
}
