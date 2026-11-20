//! Whether an mr.log call is the agent's own.

use super::LogCall;

impl LogCall {
    /// False for a call whose parent is git (a guard hook running `mortar guard-hook`) or "drive" (the drive's own
    /// conflicting change, and anything run under it); true for every other call. A predicate read as "is from the
    /// agent", not a constructor, so clippy's `from_*` naming rule does not apply.
    #[allow(clippy::wrong_self_convention)]
    pub fn from_agent(&self) -> bool {
        self.parent != "git" && self.parent != "drive"
    }
}
