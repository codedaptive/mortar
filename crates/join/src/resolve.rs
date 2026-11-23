//! resolve: continues a join that stopped on a conflict; resolve_on_base without the base it answers. It takes
//! no lock: the caller holds join-<zone> and has run recover.
//! No unit test: it only drops the base from resolve_on_base's answer; resolve_on_base's header lists the cases.

use crate::{resolve_on_base, JoinAnswer, JoinRunError};
use gitio::{Git, Oid};
use std::path::Path;

/// Resolves the join paused in `folder` exactly as resolve_on_base does, taking `tree`, the folder's tree, from the
/// caller, and answers its JoinAnswer alone.
pub fn resolve(git: &Git, folder: &Path, tree: &Oid, actor: &str) -> Result<JoinAnswer, JoinRunError> {
    resolve_on_base(git, folder, tree, actor).map(|(answer, _)| answer)
}
