//! SquashError as a std error: its source is the wrapped git, op log, trailer or change ID error; the refusals
//! squash and read_squash make themselves have none.

use crate::squash::SquashError;

impl std::error::Error for SquashError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SquashError::Git(e) => Some(e),
            SquashError::Oplog(e) => Some(e),
            SquashError::Trailers(e) => Some(e),
            SquashError::ChangeId(e) => Some(e),
            SquashError::RefMissing { .. }
            | SquashError::AttrSourceMissing { .. }
            | SquashError::Unrelated { .. }
            | SquashError::OpFailed(_)
            | SquashError::OpRepeated(_)
            | SquashError::TrailerMissing { .. }
            | SquashError::TrailerMalformed { .. }
            | SquashError::RequestsMalformed { .. } => None,
        }
    }
}
