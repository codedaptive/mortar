//! Step 1's checks 7 and 8: a squash made and checked for merging, after r3 joins zone b for check 14. Real
//! repositories and processes: run_flow.rs runs it on every drive.

use super::{not_run, requests_missing, FlowState};
use crate::report::Checks;
use crate::stage::{git_out, harness_mr, stage_join};
use crate::world::World;

/// Why check 8's conflict half runs after check 10 (flow_land.rs) rather than here.
const CONFLICT_LATER: &str = "check 8's conflict half runs after check 10: until a landing, integration sits at the \
    base commit, the repository's root, which every commit descends from, so no commit can conflict with it \
    (would-merge merges into integration's tip: land/src/squash/would_merge.rs)";

/// First r3 edits src/b/b.txt and joins zone b (a log line; check 14 judges it). Check 7: `mr squash a`: exit 0
/// "squashed" with a change ID and a commit whose one parent is origin's `main` and whose message names r1 and r2.
/// Check 8: `mr would-merge <change ID>`: exit 0 "merges" for that squash; then a log line saying the conflict half
/// runs after check 10 (CONFLICT_LATER). False, with the later checks written as not run, when no squash was made.
pub fn flow_squash(world: &World, checks: &mut Checks, state: &mut FlowState) -> bool {
    let r3 = stage_join(world, &state.r3, "src/b/b.txt", "b from r3\n");
    state.r3_joined = r3.is(0, "joined");
    state.r3_saw = r3.saw;
    checks.log(&format!("r3 edits src/b/b.txt and runs mr join, for check 14: {}", state.r3_saw));
    let squash = harness_mr(world, &["squash", "a"]);
    let (commit, change_id) = (squash.text("commit"), squash.text("change_id"));
    let parents = git_out(world, &world.work, &["rev-list", "--parents", "-n", "1", &commit]);
    let main = git_out(world, &world.work, &["rev-parse", "main"]);
    let message = git_out(world, &world.work, &["log", "-1", "--format=%B", &commit]);
    let missing = requests_missing(&message, &["r1", "r2"]);
    let made = squash.is(0, "squashed") && !commit.is_empty();
    let on_origin = parents == format!("{commit} {main}");
    let ok = made && !change_id.is_empty() && on_origin && missing.is_empty();
    let saw = format!(
        "{}; change {change_id}, commit {commit}; parents [{parents}], main {main}; message {message:?}; requests \
         not named [{}]",
        squash.saw,
        missing.join(", ")
    );
    checks.check("7. mr squash a --actor accept", ok, &saw);
    if !made {
        not_run(checks, 8, "check 7 failed");
        return false;
    }
    (state.change_id, state.first) = (change_id, commit);
    let merges = harness_mr(world, &["would-merge", &state.change_id]);
    let ok = merges.is(0, "merges") && merges.text("commit") == state.first && merges.text("change_id") == state.change_id;
    let saw = format!("{}; change {}, commit {}", merges.saw, merges.text("change_id"), merges.text("commit"));
    checks.check("8. mr would-merge <change ID> --actor accept", ok, &saw);
    checks.log(CONFLICT_LATER);
    true
}
