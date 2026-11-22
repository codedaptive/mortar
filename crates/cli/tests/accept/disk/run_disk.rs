//! Step 5, disk. Real repositories, processes and files: the drive (steps/disk.rs) runs it as step 5's body on every
//! drive with ACCEPT_MOOT_COPY set and not empty whose throwaway world was made.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{
    build_number_lines, disk_init, disk_number_lines, kit_number_lines, measure_zone, parse_ls_tree, plan_checkout,
    writable_copy, CheckoutPlan, DiskNumbers, ZoneNumbers, ZonePlan,
};
use crate::report::Checks;
use crate::stage::git_out;
use crate::world::World;

const COPY_CHECK: &str = "the copy is one the drive may write (not a registered team repository or a worktree of one)";

/// The refusal first (writable_copy): a copy the drive may not write, or one it cannot tell, is that one failed
/// line and nothing more: no mr run, nothing written, nothing measured. Otherwise the plan from HEAD's tree on a log
/// line, then disk_init's check line for the all-packages zone alone. A plan not read, or an init line that did not
/// hold, ends the step there with every number not taken, saying why: the five check lines, the all-packages
/// zone's build line and the one-kit zone's line. One run name, `accept-disk-<unix seconds>-<pid>`, is the
/// all-packages zone and its request, and `<run>-kit` the one-kit zone and its request, so no run needs a branch an
/// earlier run's folder still holds (its worktree entry outlives its removed scratch). When the init line held: the
/// all-packages zone through measure_zone on check lines, its five number lines and its build's log lines; then
/// measure_kit and the kit's number lines; then a log line naming what stays in the copy, which gets no plain-git
/// write.
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
    let plan = match copy_plan(world, &copy) {
        Ok(plan) => plan,
        Err(why) => {
            let numbers = not_taken(Err(format!("the copy's plan was not read: {why}")), "the copy's plan was not read");
            return not_measured(checks, &numbers, "the all-packages zone", "the copy's plan was not read");
        }
    };
    checks.log(&plan_line(&plan));
    let seconds = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |since| since.as_secs());
    let run = format!("accept-disk-{seconds}-{}", std::process::id());
    let label = format!("the all-packages zone ({})", plan.zone.zone_dir);
    if !disk_init(world, checks, &copy, &plan, &run, None) {
        let numbers = not_taken(Ok(plan.bytes), "the mr init and mr folder lines did not all hold");
        return not_measured(checks, &numbers, &label, "the mr init line did not hold");
    }
    let head = git_out(world, &copy, &["rev-parse", "HEAD"]);
    let at = (copy.as_path(), head.as_str());
    let zone = measure_zone(world, checks, at, [&run, "zone-disk", "request-disk"], &plan.zone, true, &label);
    let numbers = DiskNumbers {
        checkout: Ok(plan.bytes),
        zone_folder: zone.zone_folder.clone(),
        request_folder: zone.request_folder.clone(),
        store: zone.store.clone(),
        snapshot: zone.snapshot.clone(),
    };
    disk_number_lines(&numbers, checks);
    build_number_lines(&label, &zone, checks);
    let kit = measure_kit(world, checks, at, &plan, &run);
    kit_number_lines(&kit, checks);
    checks.log(&format!(
        "left in the copy: the mortar config and, of what this run made, the branches zone/{run}, req/{run}/{run}, \
         zone/{run}-kit and req/{run}-kit/{run}-kit, the snapshot refs, and the worktree entries (prunable once the \
         scratch is removed)"
    ));
}

/// The one-kit zone. Runs on every drive whose all-packages init line held, after that zone was measured. When the
/// plan has no kit it runs nothing and returns why. Else disk_init with the kit, on a log line, registers the zone
/// `<run>-kit`; when that held, measure_zone takes the kit's numbers on log lines; when it did not, Err saying so.
fn measure_kit(
    world: &World,
    checks: &mut Checks,
    at: (&Path, &str),
    plan: &CheckoutPlan,
    run: &str,
) -> Result<(String, ZoneNumbers), String> {
    let kit = plan.kit.as_ref().map_err(Clone::clone)?;
    if !disk_init(world, checks, at.0, plan, run, Some(&kit.zone_dir)) {
        return Err("the mr init line registering the kit zone did not hold".to_string());
    }
    let label = format!("the one-kit zone ({})", kit.zone_dir);
    let kit_run = format!("{run}-kit");
    let names = [kit_run.as_str(), "zone-kit", "request-kit"];
    Ok((kit.zone_dir.clone(), measure_zone(world, checks, at, names, kit, false, &label)))
}

/// An early exit's lines: the five check lines from `numbers`, then the all-packages zone's build and the one-kit
/// zone, each one line not taken because of `why`.
fn not_measured(checks: &mut Checks, numbers: &DiskNumbers, label: &str, why: &str) {
    disk_number_lines(numbers, checks);
    let untaken = ZoneNumbers {
        zone_folder: Err(why.to_string()),
        build: Err(why.to_string()),
        request_folder: Err(why.to_string()),
        built_request: Err(why.to_string()),
        store: Err(why.to_string()),
        snapshot: Err(why.to_string()),
    };
    build_number_lines(label, &untaken, checks);
    kit_number_lines(&Err(why.to_string()), checks);
}

/// `git ls-tree -r -l -z HEAD` in the copy, read by parse_ls_tree and planned by plan_checkout.
fn copy_plan(world: &World, copy: &Path) -> Result<CheckoutPlan, String> {
    let ran = world.git(copy, &["ls-tree", "-r", "-l", "-z", "HEAD"]);
    if ran.code != Some(0) {
        return Err(format!("git ls-tree -r -l -z HEAD failed: {}", ran.seen()));
    }
    plan_checkout(&parse_ls_tree(&ran.stdout))
}

/// Both zones, each with its folders and its build command, or why the plan has no kit or no build.
fn plan_line(plan: &CheckoutPlan) -> String {
    let kit = plan.kit.as_ref().map_or_else(|why| format!("none, {why}"), zone_text);
    format!("the plan: the all-packages zone: {}; the one-kit zone: {kit}", zone_text(&plan.zone))
}

fn zone_text(zone: &ZonePlan) -> String {
    let build = match &zone.build {
        Ok(build) if build.dir.is_empty() => format!("{} {} at the zone folder's top", build.program, build.args.join(" ")),
        Ok(build) => format!("{} {} in {}", build.program, build.args.join(" "), build.dir),
        Err(why) => format!("none, {why}"),
    };
    format!("{}, request folder {}, edit file {}, build {build}", zone.zone_dir, zone.request_dir, zone.edit_file)
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
