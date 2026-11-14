//! Git::push: set or delete refs on a remote, each under its own lease. It only runs git, so it has no unit tests;
//! tests/land_ops/push.rs covers it ("push to a local bare remote: new, up to date, forced under a matching lease,
//! stale lease, non-fast-forward unleased, delete, pre-receive hook refusal, results in request order, MORTAR_OP from
//! Git.env seen by the hook").

use crate::push_request::push_request;
use crate::read_push::read_push;
use crate::{Git, GitError, PushOutcome, PushRef};

impl Git {
    /// Pushes every ref in `refs` to `remote` (a configured name, a path or a URL) in one run. The answer holds one
    /// outcome per ref, in the order of `refs`; a ref git refused is an outcome, not an error. A push git refused
    /// before sending any ref (a local pre-push hook, or a source it could not use) is an error carrying git's
    /// stderr. `self.env` reaches git's hooks, local and remote, so a caller's MORTAR_OP rides on it.
    pub fn push(&self, remote: &str, refs: &[PushRef]) -> Result<Vec<PushOutcome>, GitError> {
        let (args, opts) = push_request(remote, refs)?;
        read_push(&self.run(&args, &opts)?, refs)
    }
}
