//! A request's attempts, read from the repository's refs and worktrees.

use gitio::{Oid, Worktree};

use super::Attempt;
use crate::plan::attempt_number;

/// The refs in `refs` (list_refs' shape: id, then full refname) named exactly
/// `refs/heads/try/<zone>/<request>/<k>`, with k in canonical decimal (no sign, no leading zero, nothing after
/// it) from 1 to u32::MAX, sorted by k. Each attempt's folder is the path of the worktree in `worktrees` whose
/// branch is that full name. Every other ref is ignored.
pub fn attempts_of(zone: &str, request: &str, refs: &[(Oid, String)], worktrees: &[Worktree]) -> Vec<Attempt> {
    let prefix = format!("refs/heads/try/{zone}/{request}/");
    let mut attempts: Vec<Attempt> = refs
        .iter()
        .filter_map(|(tip, name)| {
            Some(Attempt {
                k: attempt_number(name.strip_prefix(&prefix)?)?,
                branch: name.clone(),
                tip: tip.clone(),
                folder: worktrees
                    .iter()
                    .find(|w| w.branch.as_deref() == Some(name.as_str()))
                    .map(|w| w.path.clone()),
            })
        })
        .collect();
    attempts.sort_by_key(|a| a.k);
    attempts
}
