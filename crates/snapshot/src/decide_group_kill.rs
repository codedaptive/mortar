//! decide_group_kill: whether this poll signals the formatter's process group.

use crate::formatter_wait::FormatterWait;
use crate::leader_state::LeaderState;

/// True when the group has not been signalled yet and either the leader has exited but is unreaped (any step: its
/// strays are killed before the snapshot is taken), or the leader is still running at the limit (`step` Kill). A
/// reaped leader is never a reason to signal: its group id may already belong to another process.
pub(crate) fn decide_group_kill(leader: LeaderState, group_killed: bool, step: FormatterWait) -> bool {
    if group_killed {
        return false;
    }
    match leader {
        LeaderState::ExitedUnreaped => true,
        LeaderState::Running => step == FormatterWait::Kill,
        LeaderState::Reaped => false,
    }
}
