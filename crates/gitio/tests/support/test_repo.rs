use gitio::Git;
use std::path::PathBuf;

/// An exclusively owned throwaway repository, removed when the fixture drops.
pub struct TestRepo {
    pub path: PathBuf,
    pub git: Git,
}
