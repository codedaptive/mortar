//! Display for WithdrawError: each variant says why and names what it carries; the refusals withdraw makes itself
//! also say that nothing was moved and what to do next.

use crate::withdraw::WithdrawError;
use std::fmt;

impl fmt::Display for WithdrawError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WithdrawError::Squash(e) => write!(f, "withdraw: {e}"),
            WithdrawError::Git(e) => write!(f, "withdraw git: {e}"),
            WithdrawError::Oplog(e) => write!(f, "withdraw op log: {e}"),
            WithdrawError::State(e) => write!(f, "withdraw: {e}"),
            WithdrawError::Published { zone, commit, origin } => write!(
                f,
                "squash {commit} of zone {zone} is already in origin (at {origin}), so it was published and cannot \
                 be withdrawn; nothing was moved; next: commit new work on the zone, then mr squash {zone} starts a \
                 new change"
            ),
            WithdrawError::Landing { zone, commit, change_id } => write!(
                f,
                "squash {commit} of zone {zone} is the landing in flight ({change_id}), so it cannot be withdrawn; \
                 nothing was moved; next: mr back-out, then mr withdraw {zone}"
            ),
            WithdrawError::OpFailed(record) => write!(
                f,
                "op {} (key {}) failed: the squash ref was not at the commit read, so nothing was moved; run again \
                 to read it afresh",
                record.op_id, record.key
            ),
            WithdrawError::OpRepeated(record) => write!(
                f,
                "op key {} is already done in the op log, so this attempt moved nothing; run again",
                record.key
            ),
        }
    }
}
