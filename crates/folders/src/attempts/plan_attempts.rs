//! The attempt folders make_attempts makes for a request.

use gitio::Oid;

use crate::plan::{attempt_branch, request_branch, PlanError};
use crate::record::{FolderKind, FolderRecord};

use super::PlannedAttempt;

/// Attempts 1 to `n`, in order, each on attempt_branch(zone, request, k) and recorded as kind Attempt(k), cut from
/// `base` on the request branch (its parent), with `cone` as its work-order paths and no needed paths. Refuses a
/// zone or request name check_name refuses, and n == 0.
pub fn plan_attempts(
    zone: &str,
    request: &str,
    base: &Oid,
    cone: &[String],
    n: u32,
) -> Result<Vec<PlannedAttempt>, PlanError> {
    let parent = request_branch(zone, request)?;
    if n == 0 {
        return Err(PlanError {
            reason: "attempt count 0; make at least one".to_string(),
        });
    }
    (1..=n)
        .map(|k| {
            Ok(PlannedAttempt {
                k,
                branch: attempt_branch(zone, request, k)?,
                record: FolderRecord {
                    kind: FolderKind::Attempt(k),
                    zone: zone.to_string(),
                    request: Some(request.to_string()),
                    base: base.clone(),
                    work_order_paths: cone.to_vec(),
                    needed_paths: Vec::new(),
                    parent: Some(parent.clone()),
                },
            })
        })
        .collect()
}
