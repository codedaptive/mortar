//! Step 5's zone and request folders, made by Mortar from the moot copy. Real repositories and processes: the drive
//! (steps/disk.rs) runs it through run_disk on every drive that passes the refusal and reads the plan.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::{actor_mr, CheckoutPlan};
use crate::report::Checks;
use crate::stage::git_out;
use crate::world::World;

/// One run name, `accept-disk-<unix seconds>-<pid>`, is both the zone and the request, so no run needs a branch an
/// earlier run's folder still holds (its worktree entry outlives its removed scratch). In the copy, each on its own
/// check line and each only after the one before held: `mr init --replace` with the copy's branch as origin and the
/// run's zone on the plan's zone folder, every top-level file a root file (ok when "written", "already-done" or
/// "replaced"; the reply names the keys changed); `mr folder zone` at <root>/zone-disk from HEAD; `mr folder
/// request` at <root>/request-disk with the plan's request folder as its path. Then a log line names what stays in
/// the copy. Both folders sit under world.root, the step's own scratch, which run_step removes after the body; the
/// copy itself gets no plain-git write. The two folders when all three held, else None.
pub fn disk_folders(
    world: &World,
    checks: &mut Checks,
    copy: &Path,
    plan: &CheckoutPlan,
) -> Option<(PathBuf, PathBuf)> {
    let seconds = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |since| since.as_secs());
    let run = format!("accept-disk-{seconds}-{}", std::process::id());
    if !init(world, checks, copy, plan, &run) {
        return None;
    }
    let head = git_out(world, copy, &["rev-parse", "HEAD"]);
    let zone = world.root.join("zone-disk");
    let zone_text = zone.to_string_lossy().into_owned();
    let made = actor_mr(world, copy, &["folder", "zone", &run, &zone_text, "--start", &head]);
    let did = format!("mr folder zone {run} <root>/zone-disk --start {head} --actor accept");
    if !checks.check(&did, made.is(0, "made"), &made.saw) {
        return None;
    }
    let request = world.root.join("request-disk");
    let request_text = request.to_string_lossy().into_owned();
    let made = actor_mr(world, copy, &["folder", "request", &run, &run, &request_text, "--paths", &plan.request_dir]);
    let paths = &plan.request_dir;
    let did = format!("mr folder request {run} {run} <root>/request-disk --paths {paths} --actor accept");
    if !checks.check(&did, made.is(0, "made"), &made.saw) {
        return None;
    }
    checks.log(&format!(
        "left in the copy: the mortar config, the branches zone/{run} and req/{run}/{run}, the snapshot ref, and the \
         two worktree entries (prunable once the scratch is removed)"
    ));
    Some((zone, request))
}

/// `mr init --replace` in the copy on its own line; a detached HEAD fails the line before mr runs.
fn init(world: &World, checks: &mut Checks, copy: &Path, plan: &CheckoutPlan, run: &str) -> bool {
    let did = format!(
        "mr init --origin <the copy's branch> --integration integration --remote origin --zone {run}={} \
         --root-file for each of {} top-level files --replace --actor accept",
        plan.zone_dir,
        plan.root_files.len()
    );
    let branch = world.git(copy, &["symbolic-ref", "--short", "HEAD"]);
    if branch.code != Some(0) {
        return checks.check(&did, false, "the copy's HEAD is detached; check out a branch");
    }
    let zone = format!("{run}={}", plan.zone_dir);
    let mut args = vec!["init", "--origin", branch.stdout.trim(), "--integration", "integration", "--remote", "origin"];
    args.extend(["--zone", zone.as_str()]);
    for file in &plan.root_files {
        args.extend(["--root-file", file.as_str()]);
    }
    args.push("--replace");
    let reply = actor_mr(world, copy, &args);
    let ok = ["written", "already-done", "replaced"].iter().any(|outcome| reply.is(0, outcome));
    checks.check(&did, ok, &reply.saw)
}
