//! classify_fetch_ref: read the result of `git fetch` for one ref as an outcome.

use crate::{FetchRefOutcome, GitError, Output};

/// Ok(Fetched) when git succeeded. Exit 128 with stderr exactly `fatal: couldn't find remote ref <src>\n` (git
/// 2.56's wording for a source the remote does not have) is Ok(MissingSource). Any other error, such as exit 128 for
/// a remote that is not a repository, is returned unchanged with git's code and stderr. The whole stderr is matched
/// so that a different failure naming the same ref is never read as a missing source.
pub(crate) fn classify_fetch_ref(result: Result<Output, GitError>, src: &str) -> Result<FetchRefOutcome, GitError> {
    match result {
        Ok(_) => Ok(FetchRefOutcome::Fetched),
        Err(err) if err.code == Some(128) && err.stderr == format!("fatal: couldn't find remote ref {src}\n") => {
            Ok(FetchRefOutcome::MissingSource)
        }
        Err(err) => Err(err),
    }
}
