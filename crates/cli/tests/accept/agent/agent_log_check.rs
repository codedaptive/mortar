//! Step 4's check 2: the op log names the agent. Run by run_agent.rs on every drive with ACCEPT_AGENT set.

use super::{actor_kinds, AGENT_CHECKS};
use crate::report::Checks;
use crate::stage::Reply;

/// From `log` (the reply of `mr log --json`, which must be exit 0 "listed"), the agent's actor a/r1 finished a
/// "save", a join ("join-fast-forward" or "join-rebase") and a resolve (its final join-rebase, which actor_kinds
/// reads as "resolve (join-rebase)").
pub fn agent_log_check(checks: &mut Checks, log: &Reply) {
    let kinds = actor_kinds(&log.report, "a/r1");
    let has = |wanted: fn(&str) -> bool| kinds.iter().any(|kind| wanted(kind));
    let ok = log.is(0, "listed") && has(|kind| kind == "save") && has(|kind| kind.starts_with("join-"))
        && has(|kind| kind == "resolve (join-rebase)");
    checks.check(AGENT_CHECKS[2], ok, &format!("{}; a/r1's done ops [{}]", log.saw, kinds.join(", ")));
}
