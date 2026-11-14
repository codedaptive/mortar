//! JoinState: what a conflicted join leaves for resolve in mortar-join.json. A declaration only;
//! after_replay_tests, encode_join_state_tests and decode_join_state_tests exercise it.

use gitio::Oid;
use std::path::PathBuf;

/// The request ref and its tip now (resolve's later op needs it as the old value), the zone tip replayed
/// onto, and the stopped replay: the tip so far, the commit that did not apply, its conflicted tree, the
/// commits still to replay (oldest first) and the conflicted files. `files_written` is true once resolve has
/// written the conflicted files into the folder, so a resolve run tells its first run from a next one.
/// `rebased_to` is the new request tip once a resolve's continued replay is clean, saved before the op that
/// moves the request ref, so a repeated resolve can tell whether that op was applied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct JoinState {
    pub(crate) request_ref: String,
    pub(crate) request_tip: Oid,
    pub(crate) onto: Oid,
    pub(crate) replayed_tip: Oid,
    pub(crate) commit: Oid,
    pub(crate) tree: Oid,
    pub(crate) remaining: Vec<Oid>,
    pub(crate) files: Vec<PathBuf>,
    pub(crate) files_written: bool,
    pub(crate) rebased_to: Option<Oid>,
}
