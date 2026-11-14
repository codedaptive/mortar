//! PublishError: why publish was refused. A bare type (S-7);
//! publish_error_display_tests exercises the PushRefused and RemoteMissing texts, tests/publish_zit (state_display)
//! the State text, publish_error_source_tests its source.

use crate::landing::{LandError, LandingStateError};
use crate::squash::SquashError;
use gitio::{GitError, PushRefusal};
use oplog::OplogError;

/// A publish refusal. Debug only: SquashError, LandError, OplogError and LandingStateError are neither Clone nor Eq.
#[derive(Debug)]
pub enum PublishError {
    /// Reading the local origin tip or the fetched origin ref failed.
    Squash(SquashError),
    /// An op moving the local origin, integration or the squash ref was refused or failed, or so was the back-out.
    Land(LandError),
    /// A git run failed: reading the remote, pushing, fetching or reading a ref.
    Git(GitError),
    /// Writing the push's intent or closing record to the op log failed.
    Oplog(OplogError),
    /// The landing state could not be read or removed.
    State(LandingStateError),
    /// The remote refused the push for a reason other than its origin branch having moved; the remote's branch is
    /// still at the landing's starting origin.
    PushRefused { refname: String, refusal: PushRefusal },
    /// The remote has no origin branch to publish onto.
    RemoteMissing { remote: String, refname: String },
}
