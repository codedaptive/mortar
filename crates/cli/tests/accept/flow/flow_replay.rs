//! Step 1's check 14: the other zone replayed onto the new origin. Real repositories and processes: run_flow.rs
//! runs it on every drive.

use super::{one_parent_each, FlowState};
use crate::report::Checks;
use crate::stage::{git_out, harness_mr, is_ancestor};
use crate::world::World;

/// r3 joined zone b before the squash (flow_squash.rs); `mr after-publish a --move b`: exit 0 "after-published"
/// (the spec's success), zone b's branch on the new origin holding r3's change, zone a at origin's commit, and the
/// commits zone b adds to origin ordinary, one parent each. Always true: check 15 can run whatever this found.
pub fn flow_replay(world: &World, checks: &mut Checks, state: &mut FlowState) -> bool {
    let after = harness_mr(world, &["after-publish", "a", "--move", "b"]);
    let main = git_out(world, &world.work, &["rev-parse", "main"]);
    let zone_a = git_out(world, &world.work, &["rev-parse", "-q", "--verify", "zone/a"]);
    let zone_b = git_out(world, &world.work, &["rev-parse", "-q", "--verify", "zone/b"]);
    let on_origin = is_ancestor(world, "main", "zone/b");
    let b_text = git_out(world, &world.work, &["show", "zone/b:src/b/b.txt"]);
    let parents = git_out(world, &world.work, &["rev-list", "--parents", "main..zone/b"]);
    let ordinary = one_parent_each(&parents);
    let ok = state.r3_joined
        && after.is(0, "after-published")
        && on_origin
        && b_text == "b from r3"
        && zone_a == main
        && ordinary;
    let saw = format!(
        "r3 joined: {} ({}); after-publish: {}; moved [{}]; main {main}; zone/a {zone_a}; zone/b {zone_b} on main: \
         {on_origin}; zone/b's src/b/b.txt {b_text:?}; main..zone/b [{}], one parent each: {ordinary}",
        state.r3_joined,
        state.r3_saw,
        after.saw,
        after.texts("moved").join(", "),
        parents.lines().collect::<Vec<_>>().join("; ")
    );
    checks.check("14. mr after-publish a --move b --actor accept", ok, &saw);
    true
}
