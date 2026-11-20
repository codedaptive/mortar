//! Whether a folder may be removed, from its kind, its HEAD and its branch's merge base with the zone tip.

use gitio::Oid;

use super::RemoveStep;
use crate::record::FolderKind;

/// A zone folder is ZoneFolder, whatever else holds. Any other kind is judged as a request folder (remove answers
/// AttemptFolder for an attempt record before it asks). HEAD on `head_branch`, not `expected` (both full
/// `refs/heads/...` names), is OffBranch. The branch at `tip` is contained in the zone when the merge base of `tip`
/// and the zone tip is `tip` itself (the zone tip descends from it): Go. Any other merge base, or none, is NotJoined.
pub(crate) fn remove_step_for(
    kind: FolderKind,
    expected: &str,
    head_branch: &str,
    tip: &Oid,
    merge_base: Option<&Oid>,
) -> RemoveStep {
    if kind == FolderKind::Zone {
        return RemoveStep::ZoneFolder;
    }
    if head_branch != expected {
        return RemoveStep::OffBranch;
    }
    if merge_base == Some(tip) {
        return RemoveStep::Go;
    }
    RemoveStep::NotJoined
}
