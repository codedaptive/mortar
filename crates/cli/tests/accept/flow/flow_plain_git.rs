//! Step 1's check 15: plain git throughout. Real repositories and processes: run_flow.rs runs it on every drive.

use super::{worktrees_missing, FlowState};
use crate::report::Checks;
use crate::world::World;

/// After the flow, in the working repository: `git fsck --no-dangling` exits 0 and prints nothing (objects nothing
/// keeps, such as would-merge's trial trees, are not faults), and `git worktree list --porcelain` shows the two
/// zone folders and r1-r4 as worktrees.
pub fn flow_plain_git(world: &World, checks: &mut Checks, state: &FlowState) {
    let fsck = world.git(&world.work, &["fsck", "--no-dangling", "--no-progress"]);
    let clean = fsck.code == Some(0) && fsck.stdout.trim().is_empty() && fsck.stderr.trim().is_empty();
    let list = world.git(&world.work, &["worktree", "list", "--porcelain"]);
    let folders: Vec<String> = [&state.stage.zone_a, &state.stage.zone_b, &state.r1, &state.r2, &state.r3, &state.r4]
        .iter()
        .map(|folder| folder.to_string_lossy().into_owned())
        .collect();
    let missing = worktrees_missing(&list.stdout, &folders);
    let ok = clean && list.code == Some(0) && missing.is_empty();
    let saw = format!(
        "fsck: {}; output {:?}; worktree list: {}; folders not listed [{}]",
        fsck.seen(),
        format!("{}{}", fsck.stdout, fsck.stderr).trim(),
        list.seen(),
        missing.join(", ")
    );
    checks.check("15. git fsck --no-dangling, git worktree list --porcelain in work", ok, &saw);
}
