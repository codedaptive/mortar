//! How a culprit failure reads: the commit that is not a landing with its parent count, or the trailer error.

use std::fmt;

use super::CulpritError;

impl fmt::Display for CulpritError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CulpritError::NotALanding { commit, parents } => write!(
                f,
                "{commit} is not a landing: it has {parents} parents, and a landing has one (a fast-forward) \
                 or two (a merge, S-58)"
            ),
            CulpritError::Trailers(e) => write!(f, "{e}"),
        }
    }
}
