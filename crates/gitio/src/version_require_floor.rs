//! Version::require_floor: refuse a git too old for mortar.

use crate::{GitError, Version};

/// 2.46.0: the reference-transaction hook is given symbolic-ref writes (as `ref:<target>`), which land's guard
/// reads to refuse them on the guarded refs; the builds checked from 2.42.0 to 2.45.4 pass none to the hook. mortar
/// also runs `merge-tree --write-tree` with `--attr-source`, which works from 2.43.7 (2.42.0 crashes).
const FLOOR: Version = Version {
    major: 2,
    minor: 46,
    patch: 0,
};

impl Version {
    /// The version itself when it is at least 2.46.0; else a refusal naming the version found.
    pub fn require_floor(self) -> Result<Version, GitError> {
        if self >= FLOOR {
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
