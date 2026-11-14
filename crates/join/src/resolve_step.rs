//! ResolveStep: what resolve does with what it read. A declaration only; decide_resolve_tests exercises it.

use crate::MarkedFile;
use gitio::Oid;
use std::path::PathBuf;

/// The one step a resolve takes, chosen by decide_resolve.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ResolveStep {
    /// There is no mortar-join.json: no join is paused in this folder.
    NothingToResolve,
    /// The request ref is not where the paused join left it (`actual` is None when the ref is gone).
    Stale { expected: Oid, actual: Option<Oid> },
    /// First run: write the conflicted tree's versions of the conflicted files into the folder.
    WriteFiles { tree: Oid, files: Vec<PathBuf> },
    /// Next run: these files still hold conflict-marker lines.
    StillMarked(Vec<MarkedFile>),
    /// Next run: build the resolved tree with the conflicted paths from the caller's `tree`, commit it and continue the
    /// replay.
    Continue { tree: Oid },
    /// The continued replay was clean and saved, but the request ref has not moved yet: move it, then remove the state.
    Rebase { from: Oid, to: Oid },
    /// The request ref already moved to the continued replay's tip: only remove the state.
    Finish { from: Oid, to: Oid },
}
