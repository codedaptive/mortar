//! The git floor check every verb but the exempt ones makes before anything else runs.
//! Unit tests: shell/git_floor_tests.

use gitio::{GitError, Version};

use super::{checks_git_floor, git_floor_report};
use crate::report::Report;

/// Ok without calling `read_version` for a verb checks_git_floor exempts; otherwise `read_version` is called once:
/// Ok when it returns a version, else Err with git_floor_report's report of its error.
pub(crate) fn git_floor(verb: &str, read_version: &dyn Fn() -> Result<Version, GitError>) -> Result<(), Report> {
    if !checks_git_floor(verb) {
        return Ok(());
    }
    read_version().map(drop).map_err(|error| git_floor_report(verb, &error))
}
