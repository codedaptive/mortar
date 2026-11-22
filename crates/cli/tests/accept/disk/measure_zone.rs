//! One of step 5's zones, made into folders and measured. Real repositories, processes, files and a clock: the drive
//! (steps/disk.rs) runs it through run_disk on every drive whose mr init check line held, once for the all-packages
//! zone, and then once for the one-kit zone when the plan has a kit and the init registering it held.

use std::path::Path;

use super::{actor_mr, reply_number, time_build, time_snapshot, tree_bytes, ZoneNumbers, ZonePlan};
use crate::report::Checks;
use crate::stage::Reply;
use crate::world::World;

/// In the copy at `head`, for the zone `zone` (its folders at `<root>/<zone_folder>` and `<root>/<request_folder>`,
/// under world.root, the step's own scratch, which run_step removes after the body), in order: `mr folder zone`
/// from `head`; the zone folder's bytes (.git and target left out); the plan's debug build, timed in the zone folder;
/// `mr folder request` with the plan's request folder as its path, made after the build so any build output Mortar
/// shares with it is in it; the request folder's bytes (.git and target left out) and, when the build was taken,
/// again with its build output (.git alone left out); `mr disk` in the copy, its lines logged as "mr disk after
/// <label>: <line>", for its store bytes; mr snapshot's wall time after a small edit to the plan's edit file. Each
/// folder line is a check line when `as_checks`, else a log line "<did>: <saw>"; one that did not hold leaves every
/// later number not taken, saying why.
pub fn measure_zone(
    world: &World,
    checks: &mut Checks,
    (copy, head): (&Path, &str),
    [zone, zone_folder, request_folder]: [&str; 3],
    plan: &ZonePlan,
    as_checks: bool,
    label: &str,
) -> ZoneNumbers {
    let zone_path = world.root.join(zone_folder);
    let zone_text = zone_path.to_string_lossy().into_owned();
    let made = actor_mr(world, copy, &["folder", "zone", zone, &zone_text, "--start", head]);
    let did = format!("mr folder zone {zone} <root>/{zone_folder} --start {head} --actor accept");
    if !folder_line(checks, as_checks, &did, &made) {
        return not_taken("the mr folder zone line did not hold");
    }
    let zone_bytes = measured(&zone_path, &[".git", "target"]);
    let build = plan.build.clone().and_then(|build| time_build(&zone_path, &build));
    let request_path = world.root.join(request_folder);
    let request_text = request_path.to_string_lossy().into_owned();
    let paths = &plan.request_dir;
    let made = actor_mr(world, copy, &["folder", "request", zone, zone, &request_text, "--paths", paths]);
    let did = format!("mr folder request {zone} {zone} <root>/{request_folder} --paths {paths} --actor accept");
    if !folder_line(checks, as_checks, &did, &made) {
        return ZoneNumbers { zone_folder: zone_bytes, build, ..not_taken("the mr folder request line did not hold") };
    }
    let request_bytes = measured(&request_path, &[".git", "target"]);
    let built_request = build.clone().and_then(|_| measured(&request_path, &[".git"]));
    let disk = actor_mr(world, copy, &["disk"]);
    for line in &disk.lines {
        checks.log(&format!("mr disk after {label}: {line}"));
    }
    let store = reply_number(&disk, "store_bytes").map_err(|why| format!("mr disk --actor accept --json: {why}"));
    ZoneNumbers {
        zone_folder: zone_bytes,
        build,
        request_folder: request_bytes,
        built_request,
        store,
        snapshot: time_snapshot(world, &request_path, &plan.edit_file),
    }
}

/// The folder line as a check line, or as a log line that decides nothing; whether `mr folder` made the folder.
fn folder_line(checks: &mut Checks, as_checks: bool, did: &str, made: &Reply) -> bool {
    let held = made.is(0, "made");
    if as_checks {
        return checks.check(did, held, &made.saw);
    }
    checks.log(&format!("{did}: {}", made.saw));
    held
}

fn measured(folder: &Path, leave_out: &[&str]) -> Result<u64, String> {
    tree_bytes(folder, leave_out).map_err(|error| format!("cannot measure {}: {error}", folder.display()))
}

/// Every number not taken because `prerequisite` failed.
fn not_taken(prerequisite: &str) -> ZoneNumbers {
    let why = prerequisite.to_string();
    ZoneNumbers {
        zone_folder: Err(why.clone()),
        build: Err(why.clone()),
        request_folder: Err(why.clone()),
        built_request: Err(why.clone()),
        store: Err(why.clone()),
        snapshot: Err(why),
    }
}
