//! Version::require_floor: refuse a git too old for mortar.

use crate::{GitError, Version};

impl Version {
    /// The version itself when it is at least 2.46.0; else a refusal naming the version found.
    pub fn require_floor(self) -> Result<Version, GitError> {
        if self >= Version::FLOOR {
            return Ok(self);
        }
        Err(GitError::refusal(
            Vec::new(),
            format!(
                "git {}.{}.{} found; mortar needs git 2.46 or newer",
                self.major, self.minor, self.patch
            ),
        ))
    }
}
