//! How the orphan refusals read: OrphanAttempt (keep or drop_attempts met attempt folders whose try branch is gone)
//! and OrphanWithoutSnapshot (remake_orphans found no copy of them), each with its next step.

use std::path::PathBuf;

/// The text of OrphanAttempt, or of OrphanWithoutSnapshot when `no_snapshot`, naming the folders at `paths` joined
/// with ", ".
pub(crate) fn orphan_error_text(paths: &[PathBuf], no_snapshot: bool) -> String {
    let folders = paths.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(", ");
    if no_snapshot {
        return format!(
            "attempt folders {folders} have lost their try branch and have no snapshot, so Mortar holds no copy of \
             them; nothing changed; next step: copy out anything wanted, then remove each folder by hand"
        );
    }
    format!(
        "attempt folders {folders} have lost their try branch; nothing changed; next step: remake_orphans (mortar \
         runs it before keep and drop), then run the same command again"
    )
}
