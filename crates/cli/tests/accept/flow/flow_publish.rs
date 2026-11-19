//! Step 1's checks 12 and 13: a publish refused when origin moved, then published. Real repositories and
//! processes: run_flow.rs runs it on every drive.

use super::{not_run, FlowState};
use crate::report::Checks;
use crate::stage::{git_out, harness_mr, remote_git, stage_edit};
use crate::world::World;

/// Check 12: land the second squash again; the second clone commits other.txt and pushes it to origin's `main`
/// itself; `mr publish`: exit 1 "origin-moved" (OriginMoved), found = that commit, next saying land
/// again; origin's `main` is that commit on the base and nothing more; integration and main are on it; the squash
/// ref still holds the second squash. Check 13: `mr land` the squash again and `mr publish`: exit 0 "published",
/// origin's `main` and the local `main` both at the landed commit. False, with the later checks written as not run,
/// when the publish was not refused or did not publish.
pub fn flow_publish(world: &World, checks: &mut Checks, state: &mut FlowState) -> bool {
    let land = harness_mr(world, &["land", &state.second]);
    checks.check("12. mr land <second squash commit> --actor accept, again", land.is(0, "landed"), &land.saw);
    let (pushed, theirs) = someone_else_pushes(world);
    let publish = harness_mr(world, &["publish"]);
    let remote = remote_git(world, &["rev-parse", "main"]);
    let count = remote_git(world, &["rev-list", "--count", "main"]);
    let integration = git_out(world, &world.work, &["rev-parse", "integration"]);
    let main = git_out(world, &world.work, &["rev-parse", "main"]);
    let squash_ref = git_out(world, &world.work, &["rev-parse", "-q", "--verify", "refs/mortar/squash/a"]);
    let refused = publish.exit == Some(1) && publish.outcome == "origin-moved";
    let others_only = remote == theirs && count == "2";
    let on_found = publish.text("found") == theirs && integration == theirs && main == theirs;
    let ok = !theirs.is_empty() && refused && publish.next.starts_with("land") && others_only && on_found
        && squash_ref == state.second;
    let saw = format!(
        "someone else: {pushed}; publish: {}; found {}; next: {}; origin's main {remote} ({count} commits); \
         integration {integration}, main {main}; refs/mortar/squash/a {squash_ref}",
        publish.saw,
        publish.text("found"),
        publish.next
    );
    checks.check("12. the second clone pushes to origin main; mr publish --actor accept", ok, &saw);
    if !refused {
        not_run(checks, 13, "check 12 failed");
        return false;
    }
    let land = harness_mr(world, &["land", &state.second]);
    let landed = land.text("integration");
    let publish = harness_mr(world, &["publish"]);
    let remote = remote_git(world, &["rev-parse", "main"]);
    let main = git_out(world, &world.work, &["rev-parse", "main"]);
    let published = publish.is(0, "published");
    let ok = land.is(0, "landed") && published && !landed.is_empty() && remote == landed && main == landed;
    let saw = format!("land: {}; landed {landed}; publish: {}; origin's main {remote}, main {main}", land.saw, publish.saw);
    checks.check("13. mr land <second squash commit>, mr publish --actor accept", ok, &saw);
    if !published {
        not_run(checks, 14, "check 13 failed");
    }
    published
}

/// The second clone writes other.txt, commits it and pushes it to origin's `main`: what it saw, and its commit
/// ("" when it made none).
fn someone_else_pushes(world: &World) -> (String, String) {
    if let Err(error) = stage_edit(&world.other, "other.txt", "someone else\n") {
        return (format!("other.txt not written: {error}"), String::new());
    }
    let steps: [&[&str]; 3] =
        [&["add", "other.txt"], &["commit", "-q", "-m", "someone else"], &["push", "-q", "origin", "HEAD:main"]];
    for args in steps {
        let ran = world.git(&world.other, args);
        if ran.code != Some(0) {
            return (format!("git {}: {}", args.join(" "), ran.seen()), String::new());
        }
    }
    let commit = git_out(world, &world.other, &["rev-parse", "HEAD"]);
    (format!("pushed {commit}"), commit)
}
