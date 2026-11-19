//! Git::range_diff: what changed between the reviewed and the new version of a series. It only runs git, so it has
//! no unit tests; tests/typed/commits_and_refs.rs `range_diff_returns_real_git_review_text_without_colour` covers an
//! amended commit between a reviewed and a new series, compared with real git's `--no-color` output, with no ESC
//! bytes. No test covers the non-UTF-8 case (bytes shown as U+FFFD).

use crate::range_diff_request::range_diff_request;
use crate::{Git, GitError, Oid};

impl Git {
    /// git's range-diff of `base..reviewed` against `base..new`, as text for a reviewer: bytes that are not UTF-8
    /// show as U+FFFD.
    pub fn range_diff(&self, base: &Oid, reviewed: &Oid, new: &Oid) -> Result<String, GitError> {
        let (args, opts) = range_diff_request(base, reviewed, new);
        Ok(String::from_utf8_lossy(&self.run(&args, &opts)?.stdout).into_owned())
    }
}
