use super::GitRepo;
use std::ops::Deref;

impl Deref for GitRepo {
    type Target = testkit::TestRepo;

    fn deref(&self) -> &Self::Target {
        &self.repository
    }
}
