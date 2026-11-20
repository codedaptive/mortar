//! PublishError as a std error: its source is the wrapped squash, landing, git, op log or landing state error.

use crate::landing::PublishError;

impl std::error::Error for PublishError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PublishError::Squash(e) => Some(e),
            PublishError::Land(e) => Some(e),
            PublishError::Git(e) => Some(e),
            PublishError::Oplog(e) => Some(e),
            PublishError::State(e) => Some(e),
            PublishError::PushRefused { .. } | PublishError::RemoteMissing { .. } | PublishError::PushIntent { .. } => {
                None
            }
        }
    }
}
