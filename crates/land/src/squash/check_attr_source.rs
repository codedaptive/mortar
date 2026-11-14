//! check_attr_source: refuse a merge whose attribute source commit does not exist.
//! Why: squash and would_merge read merge attributes with `--attr-source=<commit>`, and git exits 0 with every
//! attribute unspecified when that names no object, so a `merge=union` file would silently merge without its rule. A
//! missing attribute source is an error, never a silent run.
//! No unit test: it runs git (S-5). tests/squash_zit.rs covers it, in a throwaway repository: every would_merge
//! case and every squash case with something to squash passes through it with an existing tip.

use crate::squash::SquashError;
use gitio::{Git, Oid};

/// Ok when `commit` names a commit; AttrSourceMissing naming it when it does not; a failed git run as Git.
pub(crate) fn check_attr_source(git: &Git, commit: &Oid) -> Result<(), SquashError> {
    match git.rev_parse(commit.as_str()).map_err(SquashError::Git)? {
        Some(_) => Ok(()),
        None => Err(SquashError::AttrSourceMissing { commit: commit.clone() }),
    }
}
