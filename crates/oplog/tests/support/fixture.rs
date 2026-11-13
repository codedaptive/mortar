use crate::repository::TestRepo;
use gitio::Oid;
use std::path::PathBuf;

pub struct Fixture {
    pub repo: TestRepo,
    pub common: PathBuf,
    pub objects: [Oid; 3],
}
