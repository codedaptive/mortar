//! Version::require_floor: refuse a git too old for mortar.

use crate::{GitError, Version};

impl Version {
    /// The version itself when it is at least `Version::FLOOR`; else a refusal naming the version found and the
    /// floor's major.minor.
    pub fn require_floor(self) -> Result<Version, GitError> {
        if self >= Version::FLOOR {
            return Ok(self);
        }
        Err(GitError::refusal(
            Vec::new(),
            format!(
                "git {}.{}.{} found; mortar needs git {}.{} or newer",
                self.major,
                self.minor,
                self.patch,
                Version::FLOOR.major,
                Version::FLOOR.minor
            ),
        ))
    }
}
