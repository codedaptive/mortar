//! Step 4's line that the conflict happened. Real files: run_agent.rs runs it on every drive with ACCEPT_AGENT
//! set.

use std::fs;

use super::{actor_kinds, conflict_seen, AgentFiles, AGENT_CHECKS};
use crate::report::Checks;
use crate::stage::Reply;

/// From mr.log (conflict_seen): the drive's `mr join` in rc is logged before the agent's first `mr join` ran, and
/// the agent ran `mr resolve`; from `log` (the reply of `mr log --json`): rc's actor a/rc finished a join. One line.
pub fn agent_conflict_check(checks: &mut Checks, files: &AgentFiles, log: &Reply) {
    let order = fs::read_to_string(&files.mr_log)
        .map_err(|error| format!("mr.log unreadable: {error}"))
        .and_then(|text| conflict_seen(&text));
    let rc_kinds = actor_kinds(&log.report, "a/rc");
    let rc_joined = rc_kinds.iter().any(|kind| kind.starts_with("join-"));
    let saw = format!(
        "mr.log: {}; mr log --json: {}; a/rc's done ops [{}]",
        order.as_ref().map_or_else(String::clone, |()| "in order".to_string()),
        log.saw,
        rc_kinds.join(", ")
    );
    checks.check(AGENT_CHECKS[0], order.is_ok() && rc_joined, &saw);
}
