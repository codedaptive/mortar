//! How a bisect failure reads: what went wrong, naming what the variant holds, and what to do.

use std::fmt;

use super::BisectError;

impl fmt::Display for BisectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BisectError::OriginMissing { refname } => {
                write!(
                    f,
                    "{refname} does not exist in this repository; name an origin branch that exists here as a \
                     local branch, then bisect again"
                )
            }
            BisectError::NotOnOrigin { failed, origin_branch } => write!(
                f,
                "{failed} is not on {origin_branch}; give a failed commit that landed on {origin_branch}"
            ),
            BisectError::OffTheLine { last_pass, failed } => write!(
                f,
                "{last_pass} is not on {failed}'s first-parent line, so the range names no landings; \
                 give a last passing commit that is a landing before {failed}"
            ),
            BisectError::ProbeCouldNotRun { commit, error } => write!(
                f,
                "the test command could not be run at {commit}: {error}; fix the command, then bisect again"
            ),
            BisectError::Git(e) => write!(f, "{e}"),
            BisectError::Culprit(e) => write!(f, "{e}"),
        }
    }
}
