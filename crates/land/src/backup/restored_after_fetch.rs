//! restored_after_fetch: what one backup ref absent locally becomes once restore_backup has fetched it.
//! restored_after_fetch_tests cover it.

use crate::backup::Restored;
use gitio::{FetchRefOutcome, Oid};

/// `fetched` is what the fetched copy holds, read only after Fetched. Fetched with an id: Created at that id, not
/// the one ls-remote listed, since the backup ref may move before the fetch and only what was fetched is in the
/// store. MissingSource, or Fetched with no copy to read: Vanished, and nothing is created for it.
pub(crate) fn restored_after_fetch(refname: String, outcome: FetchRefOutcome, fetched: Option<Oid>) -> Restored {
    match (outcome, fetched) {
        (FetchRefOutcome::Fetched, Some(oid)) => Restored::Created { refname, oid },
        (FetchRefOutcome::Fetched, None) | (FetchRefOutcome::MissingSource, _) => Restored::Vanished { refname },
    }
}
