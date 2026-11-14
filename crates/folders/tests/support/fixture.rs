use gitio::{Git, Oid};
use std::path::PathBuf;
use testkit::TestRepo;

pub struct Fixture {
    pub repo: TestRepo,
    pub git: Git,
    pub head: Oid,
    pub common: PathBuf,
}
