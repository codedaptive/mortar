use gitio::{Git, Oid};
use std::path::PathBuf;
use testkit::TestRepo;

pub struct Fixture {
    pub repo: TestRepo,
    pub git: Git,
    pub common: PathBuf,
    pub objects: [Oid; 3],
}
