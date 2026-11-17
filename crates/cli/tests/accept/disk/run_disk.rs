//! Step 5, disk (05-disk.md). Real repositories, processes and files: the drive (steps/disk.rs) runs it as step 5's
//! body on every drive with ACCEPT_MOOT_COPY set and not empty.

use std::path::Path;

use super::{
    actor_mr, disk_folders, disk_number_lines, parse_ls_tree, plan_checkout, reply_number, time_snapshot, tree_bytes,
    writable_copy, CheckoutPlan, DiskNumbers,
};
use crate::report::Checks;
use crate::world::World;

const COPY_CHECK: &str = "the copy is one the drive may write (not a registered team repository or a worktree of one)";

/// The refusal first (writable_copy): a copy the drive may not write, or one it cannot tell, is that one failed
/// line and nothing more: no mr run, nothing written, nothing measured. Otherwise the plan from HEAD's tree (a plan
/// not read leaves every number not taken, saying why), then the folders, the measures and the five number lines.
pub fn run_disk(world: &World, checks: &mut Checks) {
    let copy = match writable_copy(world) {
        Ok((copy, read)) => {
            let saw = format!("{}: not registered ({read} registered repositories read)", copy.display());
            checks.check(COPY_CHECK, true, &saw);
            copy
        }
        Err(why) => {
            checks.check(COPY_CHECK, false, &why);
            return;
        }
    };
    let numbers = match copy_plan(world, &copy) {
        Ok(plan) => measure(world, checks, &copy, &plan),
        Err(why) => not_taken(Err(format!("the copy's plan was not read: {why}")), "the copy's plan was not read"),
    };
    disk_number_lines(&numbers, checks);
}

/// `git ls-tree -r -l -z HEAD` in the copy, read by parse_ls_tree and planned by plan_checkout.
fn copy_plan(world: &World, copy: &Path) -> Result<CheckoutPlan, String> {
    let ran = world.git(copy, &["ls-tree", "-r", "-l", "-z", "HEAD"]);
    if ran.code != Some(0) {
        return Err(format!("git ls-tree -r -l -z HEAD failed: {}", ran.seen()));
    }
    plan_checkout(&parse_ls_tree(&ran.stdout))
}

/// The plan on a log line, the folders (disk_folders, which also logs what stays in the copy), then, when they were
/// made, each folder's tree_bytes, `mr disk`'s store (its lines logged) and the snapshot's wall time in the request
/// folder, in that order.
fn measure(world: &World, checks: &mut Checks, copy: &Path, plan: &CheckoutPlan) -> DiskNumbers {
    checks.log(&format!(
        "the plan: zone {}, request folder {}, edit file {}",
        plan.zone_dir, plan.request_dir, plan.edit_file
    ));
    let Some((zone, request)) = disk_folders(world, checks, copy, plan) else {
        return not_taken(Ok(plan.bytes), "the mr init and mr folder lines did not all hold");
    };
    let measured = |folder: &Path| {
        tree_bytes(folder).map_err(|error| format!("cannot measure {}: {error}", folder.display()))
    };
    let zone_folder = measured(&zone);
    let request_folder = measured(&request);
    let disk = actor_mr(world, copy, &["disk"]);
    for line in &disk.lines {
        checks.log(&format!("mr disk: {line}"));
    }
    let store = reply_number(&disk, "store_bytes").map_err(|why| format!("mr disk --actor accept --json: {why}"));
    let snapshot = time_snapshot(world, &request, &plan.edit_file);
    DiskNumbers { checkout: Ok(plan.bytes), zone_folder, request_folder, store, snapshot }
}

/// `checkout` as given and every other number not taken because `prerequisite` failed.
fn not_taken(checkout: Result<u64, String>, prerequisite: &str) -> DiskNumbers {
    let why = prerequisite.to_string();
    DiskNumbers {
        checkout,
        zone_folder: Err(why.clone()),
        request_folder: Err(why.clone()),
        store: Err(why.clone()),
        snapshot: Err(why),
    }
}
