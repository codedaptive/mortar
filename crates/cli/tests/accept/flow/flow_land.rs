//! Step 1's checks 10 and 11, with check 8's conflict case run between them: landed, then backed out after a failed
//! integration test. Real repositories and processes: run_flow.rs runs it on every drive.

use super::{conflict_commit, not_run, FlowState};
use crate::report::Checks;
use crate::stage::{git_out, harness_mr, is_ancestor, remote_git};
use crate::world::World;

/// The line of check 8's conflict half, written here whether it runs or not.
const CONFLICT_CHECK: &str = "8 (run after check 10). mr would-merge <a commit made to conflict with integration>";

/// Check 10: `mr land <second squash>`: exit 0 "landed", integration at the reply's `integration` and holding the
/// squash. Then check 8's conflict case, which needs a landing first (flow_squash.rs): a commit on origin with the
/// tree zone a had after r1's edit of src/a/a.txt conflicts with the landed squash in that file. Check 11: the drive
/// declares the integration test failed and runs `mr back-out`: exit 0 "backed-out", integration and main back at
/// the base commit, origin's `main` unchanged, the squash ref still at the second squash. False, with the later
/// checks written as not run, when the land or the back-out did not do its work; a failed land also writes check
/// 8's conflict half as not run.
pub fn flow_land(world: &World, checks: &mut Checks, state: &mut FlowState) -> bool {
    let land = harness_mr(world, &["land", &state.second]);
    let integration = git_out(world, &world.work, &["rev-parse", "integration"]);
    let holds = is_ancestor(world, &state.second, "integration");
    let landed = land.is(0, "landed");
    let ok = landed && land.text("integration") == integration && holds;
    let saw = format!("{}; integration {integration} (reply {}); holds the squash: {holds}", land.saw, land.text("integration"));
    checks.check("10. mr land <second squash commit> --actor accept", ok, &saw);
    if !landed {
        checks.check(CONFLICT_CHECK, false, "not run: check 10 failed");
        not_run(checks, 11, "check 10 failed");
        return false;
    }
    let conflict = conflict_commit(world, &state.r1_edited);
    let refused = harness_mr(world, &["would-merge", &conflict]);
    let files = refused.texts("files");
    let ok = refused.exit == Some(2) && refused.outcome == "conflict" && files.iter().any(|file| file == "src/a/a.txt");
    let saw = format!("commit {conflict}; {}; files [{}]", refused.saw, files.join(", "));
    checks.check(CONFLICT_CHECK, ok, &saw);
    checks.log("the drive declares the integration test failed");
    let back = harness_mr(world, &["back-out"]);
    let integration = git_out(world, &world.work, &["rev-parse", "integration"]);
    let main = git_out(world, &world.work, &["rev-parse", "main"]);
    let remote = remote_git(world, &["rev-parse", "main"]);
    let squash_ref = git_out(world, &world.work, &["rev-parse", "-q", "--verify", "refs/mortar/squash/a"]);
    let backed = back.is(0, "backed-out");
    let base = world.base.as_str();
    let ok = backed && integration == base && main == base && remote == base && squash_ref == state.second;
    let saw = format!(
        "{}; integration {integration}, main {main}, origin's main {remote}, base {base}; refs/mortar/squash/a \
         {squash_ref}",
        back.saw
    );
    checks.check("11. mr back-out --actor accept", ok, &saw);
    if !backed {
        not_run(checks, 12, "check 11 failed");
    }
    backed
}
