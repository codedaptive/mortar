//! cap_mismatch: where the capped tree does not differ from the full tree by exactly the refused paths.

use crate::refused_file::RefusedFile;
use std::path::PathBuf;

/// `dropped` is what changed between the full tree and the capped tree, which must be exactly the refused paths.
/// Returns each refused path not in `dropped` (in refused order), then each dropped path that was not refused (in
/// dropped order). Either kind means git read an exclude differently than intended (e.g. GIT_NOGLOB_PATHSPECS or
/// GIT_LITERAL_PATHSPECS in the environment, which gitio does not clear), or the folder changed between the two
/// tree builds.
pub(crate) fn cap_mismatch(refused: &[RefusedFile], dropped: &[PathBuf]) -> Vec<PathBuf> {
    let missing = refused.iter().map(|file| &file.path).filter(|path| !dropped.contains(path));
    let extra = dropped.iter().filter(|path| !refused.iter().any(|file| &file.path == *path));
    missing.chain(extra).cloned().collect()
}
