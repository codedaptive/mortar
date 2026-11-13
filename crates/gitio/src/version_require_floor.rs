//! Version::require_floor: refuse a git too old for mortar.

use crate::{GitError, Version};

/// 2.40.0: `merge-tree --write-tree` arrived in 2.38 and `--attr-source` in 2.40; mortar uses both.
const FLOOR: Version = Version {
    major: 2,
    minor: 40,
    patch: 0,
};

impl Version {
    /// The version itself when it is at least 2.40.0; else a refusal naming the version found.
    pub fn require_floor(self) -> Result<Version, GitError> {
        if self >= FLOOR {
            return Ok(self);
        }
        Err(GitError::refusal(
            Vec::new(),
            format!(
                "git {}.{}.{} found; mortar needs git 2.40 or newer",
                self.major, self.minor, self.patch
            ),
        ))
    }
}
