//! Version::parse: read the output of `git version`.

use crate::{GitError, Version};

impl Version {
    /// Parses `git version X.Y[.Z]`; a missing patch is 0, and anything after the first space or a fourth
    /// dotted part is ignored (Apple's "(Apple Git-146)", Windows' ".windows.1"). Anything else, including a
    /// suffix on a number such as "2.40.0-rc1" or a sign such as "+2.40.0", is a refusal naming the text found.
    pub fn parse(stdout: &[u8]) -> Result<Version, GitError> {
        let refuse = || {
            GitError::refusal(
                Vec::new(),
                format!("unrecognised `git version` output: {:?}", String::from_utf8_lossy(stdout)),
            )
        };
        let text = std::str::from_utf8(stdout).map_err(|_| refuse())?;
        let number = text
            .trim()
            .strip_prefix("git version ")
            .and_then(|rest| rest.split_whitespace().next())
            .ok_or_else(refuse)?;
        let mut parts = number.split('.');
        let mut next = |required: bool| match parts.next() {
            // u32's parser accepts a leading '+'; a version part is digits only.
            Some(part) if !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()) => part.parse::<u32>().ok(),
            Some(_) => None,
            None if required => None,
            None => Some(0),
        };
        match (next(true), next(true), next(false)) {
            (Some(major), Some(minor), Some(patch)) => Ok(Version { major, minor, patch }),
            _ => Err(refuse()),
        }
    }
}
