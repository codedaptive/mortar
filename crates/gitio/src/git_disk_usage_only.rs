//! Git::disk_usage_only: the disk bytes a set of refs alone holds. The run itself has no unit test; tests/typed/
//! disk_usage.rs covers it: a ref with unique objects counts them; objects held by a parent another ref reaches are
//! not counted; a ref HEAD (or a linked worktree's HEAD) points at or contains counts 0; an unknown ref is refused
//! naming it; an empty list answers 0; the documented upper bound (other objects shared with other refs may be
//! counted).

use crate::disk_usage_only_request::disk_usage_only_request;
use crate::read_disk_usage::read_disk_usage;
use crate::{Git, GitError};

impl Git {
    /// The on-disk bytes, as `git rev-list --disk-usage` counts them (packed size for a packed object, file size for a
    /// loose one), of the objects git's walk finds reachable from `refs` and not from any other ref. An empty `refs`
    /// answers 0 without running git.
    ///
    /// "Any other ref" is spelled `--not --exclude=<each ref> --all`, so HEAD and each linked worktree's HEAD are other
    /// refs: a ref checked out somewhere, or contained in a detached HEAD, is not freed by deleting it, and its
    /// objects count 0. Reflogs are not walked (`--all` does not read them): an object only a reflog holds is
    /// counted, though git keeps it until that reflog entry expires and gc prunes it.
    ///
    /// An upper bound, accepted: an exact walk over every other ref's objects costs far more, for a figure that only
    /// guides a sweep. It is not an exact set difference: git marks as shared the trees and blobs other refs point at
    /// directly, and the trees of the other refs' commits that are parents of commits only the given refs reach, where
    /// its walk stops, with what those trees hold. An object the given refs share with other refs may still be counted
    /// when none of those holds it: a blob re-added from an older commit, or the whole tree of a parentless commit,
    /// even when another ref's tip commit has that same tree.
    ///
    /// `--disk-usage` arrived in git 2.31, below the 2.40 floor, so there is no version check.
    ///
    /// Errors: a ref that is not a usable full ref name is refused naming it before git runs (see
    /// disk_usage_only_request); an unknown ref is refused naming it; any other exit (not a repository, a corrupt
    /// ref) is a GitError with git's exit code and stderr; an exit 0 whose stdout is not one digit line fitting a u64
    /// is refused naming what was found.
    pub fn disk_usage_only<S: AsRef<str>>(&self, refs: &[S]) -> Result<u64, GitError> {
        if refs.is_empty() {
            return Ok(0);
        }
        let (args, opts) = disk_usage_only_request(refs)?;
        read_disk_usage(&self.run(&args, &opts)?, refs)
    }
}
