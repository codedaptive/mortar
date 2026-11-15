//! Display for PublishError: each variant says why and names what it carries; the refusals publish makes itself
//! also say that nothing was published.

use crate::landing::PublishError;
use gitio::PushRefusal;
use std::fmt;

impl fmt::Display for PublishError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PublishError::Squash(e) => write!(f, "publish: {e}"),
            PublishError::Land(e) => write!(f, "publish: {e}"),
            PublishError::Git(e) => write!(f, "publish git: {e}"),
            PublishError::Oplog(e) => write!(f, "publish op log: {e}"),
            PublishError::State(e) => write!(f, "publish: {e}"),
            PublishError::PushRefused { refname, refusal } => write!(
                f,
                "the remote refused the push of {refname} ({}); the remote's {refname} is still where the landing \
                 started, so nothing was published",
                why(refusal)
            ),
            PublishError::RemoteMissing { remote, refname } => write!(
                f,
                "remote {remote} has no {refname} to publish onto; nothing was published"
            ),
            PublishError::PushIntent { op_id, .. } => write!(
                f,
                "the push intent {op_id} for this move was already open or already done when publish opened it, so \
                 nothing was pushed; next: check the remote's origin branch, then mortar publish"
            ),
        }
    }
}

/// The refusal in words; Other carries git's summary verbatim.
fn why(refusal: &PushRefusal) -> String {
    match refusal {
        PushRefusal::StaleLease => "stale lease".to_string(),
        PushRefusal::NonFastForward => "not a fast-forward".to_string(),
        PushRefusal::Hook => "a hook on the remote declined it".to_string(),
        PushRefusal::Other(summary) => summary.clone(),
    }
}
