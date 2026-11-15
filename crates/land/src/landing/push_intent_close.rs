//! push_intent_close: how a push intent is closed, from whether the push landed and what the remote's origin branch
//! holds (S-103). push_intent_close_tests covers every branch.

use gitio::Oid;
use oplog::IntentClose;

/// `landed` (the remote at or past new, as push_landed answers): Done. Otherwise Failed, naming `refname` with what
/// the remote holds, absent included.
pub(crate) fn push_intent_close(remote: Option<&Oid>, landed: bool, refname: &str) -> IntentClose {
    if landed {
        return IntentClose::Done;
    }
    IntentClose::Failed { refs: vec![(refname.to_string(), remote.cloned())] }
}
