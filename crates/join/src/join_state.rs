//! JoinState: what a conflicted join leaves for resolve in mortar-join.json. A declaration only;
//! after_replay_tests, encode_join_state_tests and decode_join_state_tests exercise it.

use gitio::Oid;
use std::path::PathBuf;

/// The request ref and its tip now (resolve's later op needs it as the old value), the zone tip replayed
/// onto, and the stopped replay: the tip so far, the commit that did not apply, its conflicted tree, the
/// commits still to replay (oldest first) and this round's conflicted files. `all_files` is every conflicted path
/// of this join over all its rounds, this round's `files` included, in first-seen order. `files_written` is true
/// once resolve has written the conflicted files into the folder, so a resolve run tells its first run from a next
/// one. `rebased_to` is the new request tip once a resolve's continued replay is clean, and `folder_tree` the tree
/// the caller moves the folder from: the old request tip's tree with every path in `all_files` taken from
/// the caller's tree. Both are saved before the op that moves the request ref, so a repeated resolve can tell
/// whether that op was applied and answers the same folder_tree.
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
    pub(crate) all_files: Vec<PathBuf>,
    pub(crate) files_written: bool,
    pub(crate) rebased_to: Option<Oid>,
    pub(crate) folder_tree: Option<Oid>,
}
