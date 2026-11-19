//! Step 1, the flow. Real repositories and processes: the drive (steps/flow.rs) runs it as step 1's
//! body on every drive.

use super::{
    flow_conflict, flow_land, flow_plain_git, flow_publish, flow_rebase, flow_replay, flow_requests, flow_rereview,
    flow_squash, not_run, FlowState,
};
use crate::report::Checks;
use crate::stage::stage_new;
use crate::world::World;

/// A group of checks: false when it failed so that the later checks cannot run, having written them as not run.
type Group = fn(&World, &mut Checks, &mut FlowState) -> bool;

/// The groups in check order: checks 1-3, 4, 5-6, 7-8, 9, 10-11, 12-13, 14.
const GROUPS: [Group; 8] =
    [flow_requests, flow_rebase, flow_conflict, flow_squash, flow_rereview, flow_land, flow_publish, flow_replay];

/// The set-up (stage_new), then the fifteen checks in order, each at least one line. A failed set-up writes all
/// fifteen as not run; a group that stops writes the rest as not run, so check 15 runs only when every group went on.
pub fn run_flow(world: &World, checks: &mut Checks) {
    let Some(stage) = stage_new(world, checks) else {
        return not_run(checks, 1, "the set-up failed");
    };
    let mut state = FlowState { stage, ..FlowState::default() };
    if GROUPS.iter().all(|group| group(world, checks, &mut state)) {
        flow_plain_git(world, checks, &state);
    }
}
