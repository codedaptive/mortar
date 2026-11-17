//! AfterReplayStep: what join does after a replay. A declaration only; after_replay_tests exercises it.

use crate::join_state::JoinState;
use gitio::Oid;

/// Move the request ref from its tip to the replayed tip, or stop with the state resolve continues from.
// A short-lived step value built and matched once per join; boxing JoinState would change every construction and
// match site for no gain.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AfterReplayStep {
    Rebase { from: Oid, to: Oid },
    Pause(JoinState),
}
