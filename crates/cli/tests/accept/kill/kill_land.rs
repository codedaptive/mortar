//! 03's case 3: a land killed before, or after, its integration move. Real repositories, processes and a SIGKILL:
//! run_kill.rs runs it twice on every drive.

use super::{kill_armed, kill_consistent, kill_log_check, CaseLines, KillState};
use crate::report::Checks;
use crate::stage::{git_out, harness_mr, is_ancestor};
use crate::world::{Crash, World};

const INTEGRATION: &str = "refs/heads/integration";

/// Case `case` (2: before, 3: after; KILL_CASES' index) with `squash`, a squash on origin's tip: `mr land <squash>
/// --actor accept --json` killed at its first update-ref, integration's move: the open intent is a land; `mr land
/// <same commit>` again exits 0 "landed" naming integration's tip; integration holds the squash and is one commit
/// past origin's `main`; the log shows the killed op closed failed (not-moved) before, done (moved) after, and one
/// done land of integration since. Then the consistency items, repeating the land. True when the repeat landed.
pub fn kill_land(world: &World, checks: &mut Checks, state: &KillState, case: usize, squash: &str, crash: Crash) -> bool {
    let mut lines = CaseLines::new(case);
    let args = ["land", squash, "--actor", "accept", "--json"];
    let landed = match kill_armed(world, checks, &mut lines, (&world.work, &args), crash, "land") {
        None => {
            lines.not_run(checks, "the kill did not stop a land at its ref transaction", false);
            false
        }
        Some((killed, _)) => {
            let land = harness_mr(world, &["land", squash]);
            let tip = git_out(world, &world.work, &["rev-parse", "-q", "--verify", INTEGRATION]);
            let saw = format!("{}; reply integration {}, integration {tip}", land.saw, land.text("integration"));
            let landed = lines.check(checks, land.is(0, "landed") && land.text("integration") == tip, &saw);
            let holds = is_ancestor(world, squash, &tip);
            let past = git_out(world, &world.work, &["rev-list", "--count", "main..integration"]);
            let saw = format!("integration {tip} holds the squash {squash}: {holds}; commits past main: {past}");
            lines.check(checks, holds && past == "1", &saw);
            let closed = if crash == Crash::Before { ("failed", "not-moved") } else { ("done", "moved") };
            kill_log_check(world, checks, &mut lines, &killed, closed, INTEGRATION);
            landed
        }
    };
    kill_consistent(world, checks, &mut lines, &state.folders, &|| harness_mr(world, &["land", squash]));
    landed
}
