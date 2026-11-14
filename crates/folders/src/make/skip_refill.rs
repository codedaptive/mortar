//! skip_refill: whether a same-record repeat of make leaves a found folder as it is instead of refilling it (S-36).

use gitio::StatusPaths;

/// True exactly when `status` lists conflicted paths: an index with unmerged entries belongs to a folder that was
/// filled (a conflict happens only in a filled folder), and a refill's checkout would fail on it, so the repeat
/// returns the record and touches nothing (S-36). Any other status, clean or with changed or untracked paths,
/// gives false: the repeat refills the folder (S-26).
pub(crate) fn skip_refill(status: &StatusPaths) -> bool {
    !status.conflicted.is_empty()
}
