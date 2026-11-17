//! What of a request's attempts is left: its attempt branches and attempt folders, as remove reads them (S-82).

use gitio::{Oid, Worktree};

use super::orphan_attempts;
use crate::plan::attempt_number;

/// What attempts_of calls an attempt, as remove names it: the full names of the refs in `refs` (list_refs' shape:
/// id, then full refname) named exactly `refs/heads/try/<zone>/<request>/<k>` with attempt_number(k) Some, in input
/// order; then the display text of the path of each attempt folder whose branch is gone (orphan_attempts), in the
/// input order of `worktrees`, which keep and drop_attempts refuse (OrphanAttempt) until remake_orphans makes them
/// ordinary. Nothing else is an attempt, so nothing else is counted: a ref or worktree branch under the prefix with
/// a k that is not a canonical attempt number (`01`, `0`) or with a further '/' (`1/x`), a request that shares a
/// prefix with `request`, another zone, req/ branches and detached worktrees.
pub fn attempts_remaining(zone: &str, request: &str, refs: &[(Oid, String)], worktrees: &[Worktree]) -> Vec<String> {
    let prefix = format!("refs/heads/try/{zone}/{request}/");
    let orphans = orphan_attempts(zone, request, refs, worktrees);
    refs.iter()
        .filter(|(_, name)| name.strip_prefix(&prefix).and_then(attempt_number).is_some())
        .map(|(_, name)| name.clone())
        .chain(
            worktrees
                .iter()
                .filter(|w| orphans.iter().any(|(_, path)| *path == w.path))
                .map(|w| w.path.display().to_string()),
        )
        .collect()
}
