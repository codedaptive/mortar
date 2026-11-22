//! WithdrawError as a std error: its source is the wrapped squash, git, op log or landing state error.

use crate::withdraw::WithdrawError;

impl std::error::Error for WithdrawError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            WithdrawError::Squash(e) => Some(e),
            WithdrawError::Git(e) => Some(e),
            WithdrawError::Oplog(e) => Some(e),
            WithdrawError::State(e) => Some(e),
            WithdrawError::Published { .. }
            | WithdrawError::Landing { .. }
            | WithdrawError::OpFailed(_)
            | WithdrawError::OpRepeated(_) => None,
        }
    }
}
