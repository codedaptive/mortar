//! snapshot_ref_name: the ref a request's or an attempt's snapshots live on. The crate exports it so a caller
//! (such as the cli) can name a target's snapshot ref through it instead of copying the rule.

use crate::{SnapshotError, SnapshotTarget};

/// `refs/mortar/snap/<zone>/<request>` with no attempt, `refs/mortar/trysnap/<zone>/<request>/<k>`
/// for attempt k, k from 1 in decimal. A zone or request that is empty or holds '/' is refused with BadTarget naming
/// the field and its text, since it would change which ref is named; attempt 0 is refused with BadTarget naming the
/// field and its value, since folders numbers attempts from 1. Other ref-name checks are left to oplog apply:
/// before it writes its intent, its check_ref_names refuses with BadMove a name that is empty, holds a NUL
/// or is not under refs/ (check_ref_moves), then one that git check-ref-format refuses.
pub fn snapshot_ref_name(target: &SnapshotTarget) -> Result<String, SnapshotError> {
    for (field, text) in [("zone", &target.zone), ("request", &target.request)] {
        if text.is_empty() || text.contains('/') {
            return Err(SnapshotError::BadTarget {
                reason: format!("{field} {text:?} is empty or holds '/'"),
            });
        }
    }
    Ok(match target.attempt {
        None => format!("refs/mortar/snap/{}/{}", target.zone, target.request),
        Some(0) => return Err(SnapshotError::BadTarget { reason: "attempt 0 is not 1 or more".to_string() }),
        Some(k) => format!("refs/mortar/trysnap/{}/{}/{k}", target.zone, target.request),
    })
}
