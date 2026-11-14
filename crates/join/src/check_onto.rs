//! check_onto: refuse a replay whose onto commit does not exist.
//! Why: git's replay and merge_tree both read merge attributes with `--attr-source=<onto>`, and git exits 0 with every
//! attribute unspecified when that names no object, so a `merge=union` file would silently merge without its rule. A
//! missing onto is an error, never a silent run.

use crate::replay_git::ReplayGit;
use crate::JoinError;
use gitio::Oid;

/// Ok when `onto` names a commit; OntoMissing naming it when it does not; a failed git run as Git.
pub(crate) fn check_onto(git: &impl ReplayGit, onto: &Oid) -> Result<(), JoinError> {
    match git.rev_parse(onto.as_str())? {
        Some(_) => Ok(()),
        None => Err(JoinError::OntoMissing(onto.clone())),
    }
}
