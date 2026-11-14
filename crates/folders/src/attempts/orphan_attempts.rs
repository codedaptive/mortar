//! A request's attempt folders whose attempt branch is gone (S-69, S-82).
//! Its tests are zone integration tests in tests/folders/attempt_plans.rs: it uses plan/attempt_number.rs for real
//! (#1264).

use std::path::PathBuf;

use gitio::{Oid, Worktree};

use crate::plan::attempt_number;

/// Each worktree in `worktrees` whose branch is named exactly `refs/heads/try/<zone>/<request>/<k>` with
/// attempt_number(k) Some, and is not one of the full refnames in `refs` (list_refs' shape: id, then full refname),
/// as (k, its path), sorted by k. Detached worktrees, other requests and zones, and branches whose tail is not a
/// canonical attempt number are not attempts, so they are never listed.
pub fn orphan_attempts(zone: &str, request: &str, refs: &[(Oid, String)], worktrees: &[Worktree]) -> Vec<(u32, PathBuf)> {
    let prefix = format!("refs/heads/try/{zone}/{request}/");
    let mut orphans: Vec<(u32, PathBuf)> = worktrees
        .iter()
        .filter_map(|w| {
            let branch = w.branch.as_deref()?;
            let k = attempt_number(branch.strip_prefix(&prefix)?)?;
            (!refs.iter().any(|(_, name)| name == branch)).then(|| (k, w.path.clone()))
        })
        .collect();
    orphans.sort_by_key(|(k, _)| *k);
    orphans
}
