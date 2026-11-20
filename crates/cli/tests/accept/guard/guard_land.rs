//! Step 2's check 8: Mortar's own moves of the guarded refs go through. Real repositories and processes:
//! run_guard.rs runs it on every drive.

use super::{local_ref, shown_ref};
use crate::report::Checks;
use crate::stage::{harness_mr, is_ancestor, remote_git};
use crate::world::World;

const LAND: &str = "8. mr land <squash commit> --actor accept";
const PUBLISH: &str = "8. mr publish --actor accept";

/// `mr land <squash>`: exit 0 "landed", integration moved to the reply's `integration`, which holds the squash.
/// Then `mr publish`: exit 0 "published", origin's branch (`main`, local) and the bare remote's `main` both moved to
/// that landed commit. Each is one line; with no squash, both are failed lines naming why, and with no landing, the
/// publish is.
pub fn guard_land(world: &World, checks: &mut Checks, squash: Result<String, String>) {
    let commit = match squash {
        Ok(commit) => commit,
        Err(why) => {
            let saw = format!("not run: no squash could be made: {why}");
            checks.check(LAND, false, &saw);
            checks.check(PUBLISH, false, &saw);
            return;
        }
    };
    let before = local_ref(world, "refs/heads/integration");
    let land = harness_mr(world, &["land", &commit]);
    let landed = local_ref(world, "refs/heads/integration");
    let holds = is_ancestor(world, &commit, "integration");
    let ok = land.is(0, "landed") && landed != before && land.text("integration") == landed && holds;
    let saw = format!(
        "{}; integration {} -> {} (reply {}); holds the squash {commit}: {holds}",
        land.saw,
        shown_ref(&before),
        shown_ref(&landed),
        land.text("integration")
    );
    if !checks.check(LAND, ok, &saw) {
        checks.check(PUBLISH, false, "not run: the land did not go through");
        return;
    }
    let (main, remote) = (local_ref(world, "refs/heads/main"), remote_git(world, &["rev-parse", "main"]));
    let publish = harness_mr(world, &["publish"]);
    let (main_after, remote_after) = (local_ref(world, "refs/heads/main"), remote_git(world, &["rev-parse", "main"]));
    let moved = main_after != main && remote_after != remote;
    let ok = publish.is(0, "published") && moved && main_after == landed && remote_after == landed;
    let saw = format!(
        "{}; main {} -> {}; origin's main {} -> {}; landed {landed}",
        publish.saw,
        shown_ref(&main),
        shown_ref(&main_after),
        shown_ref(&remote),
        shown_ref(&remote_after)
    );
    checks.check(PUBLISH, ok, &saw);
}
