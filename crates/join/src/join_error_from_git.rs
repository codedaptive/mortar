//! From<GitError> for JoinError: a failed git run becomes JoinError::Git, so `?` carries it.

use crate::JoinError;
use gitio::GitError;

impl From<GitError> for JoinError {
    fn from(error: GitError) -> JoinError {
        JoinError::Git(error)
    }
}
