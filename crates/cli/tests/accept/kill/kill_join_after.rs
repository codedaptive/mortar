//! 03's case 2: a rebase join killed after its request-ref move, before its folder move (S-150). Real repositories,
//! processes and a SIGKILL: run_kill.rs runs it on every drive.

use super::{kill_armed, kill_consistent, kill_log_check, CaseLines, KillState};
use crate::report::Checks;
use crate::stage::{agent_mr, git_out, is_ancestor};
use crate::world::{Crash, World};

const REQUEST: &str = "refs/heads/req/a/r2";

/// `mr join --json` in r2, whose zone moved under it (case 1), so join
/// rebases r2 and its first update-ref is the request ref's, killed after it: the open intent is a join-rebase; the
/// request ref moved to the op's new tip, on zone a; `mr status` exits 0 "stalled", next "mr join"; `mr join` again
/// exits 0 with r2's HEAD at that tip and its status clean; the log shows the killed op closed done and one done
/// rebase of the request ref. Then the consistency items, repeating `mr join` in r2.
pub fn kill_join_after(world: &World, checks: &mut Checks, state: &KillState) {
    let mut lines = CaseLines::new(1);
    let request = || git_out(world, &world.work, &["rev-parse", "-q", "--verify", REQUEST]);
    let before = request();
    let armed = kill_armed(world, checks, &mut lines, (&state.r2, &["join", "--json"]), Crash::After, "join-rebase");
    match armed {
        None => lines.not_run(checks, "the kill did not stop a join at its ref transaction", false),
        Some((killed, _)) => {
            let moved = request();
            let planned = killed.moves.iter().find(|(name, _)| name == REQUEST).map(|(_, new)| new.as_str());
            let on_zone = is_ancestor(world, "zone/a", &moved);
            let ok = moved != before && planned == Some(moved.as_str()) && on_zone;
            let saw = format!("req/a/r2 {before} -> {moved}; the op's new tip {planned:?}; on zone/a: {on_zone}");
            lines.check(checks, ok, &saw);
            let status = agent_mr(world, &state.r2, &["status"]);
            let saw = format!(
                "{}; next: {}; stalled_from {}, stalled_to {}; lines {:?}",
                status.saw,
                status.next,
                status.text("stalled_from"),
                status.text("stalled_to"),
                status.lines
            );
            lines.check(checks, status.is(0, "stalled") && status.next == "mr join", &saw);
            let join = agent_mr(world, &state.r2, &["join"]);
            let head = git_out(world, &state.r2, &["rev-parse", "HEAD"]);
            let changes = git_out(world, &state.r2, &["status", "--porcelain"]);
            let ok = join.exit == Some(0) && head == moved && changes.is_empty();
            let saw = format!("{}; lines {:?}; r2 HEAD {head}, status {changes:?}", join.saw, join.lines);
            lines.check(checks, ok, &saw);
            kill_log_check(world, checks, &mut lines, &killed, ("done", "moved"), REQUEST);
        }
    }
    kill_consistent(world, checks, &mut lines, &state.folders, &|| agent_mr(world, &state.r2, &["join"]));
}
