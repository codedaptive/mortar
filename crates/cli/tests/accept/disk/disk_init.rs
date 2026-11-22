//! Step 5's `mr init` in the repository copy. Real repositories and processes: the drive (steps/disk.rs) runs it
//! through run_disk on every drive that passes the refusal and reads the plan, without a kit; and once more, with
//! the kit, only after that first line held and the all-packages zone was measured, when the plan has a kit.

use std::path::Path;

use super::{actor_mr, CheckoutPlan};
use crate::report::Checks;
use crate::world::World;

/// `mr init --replace` in the copy, ok when its outcome is "written", "already-done" or "replaced" (the reply names
/// the keys changed): the copy's branch as origin, the zone `run` on the all-packages zone's folder, every top-level
/// file a root file. With `kit` None, that on one check line. With `kit` Some(<kit dir>), the same arguments plus the
/// zone `<run>-kit` on that folder, on a log line "<did>: <saw>" that decides nothing. A detached HEAD fails the
/// line before mr runs. Returns whether the init held.
pub fn disk_init(
    world: &World,
    checks: &mut Checks,
    copy: &Path,
    plan: &CheckoutPlan,
    run: &str,
    kit: Option<&str>,
) -> bool {
    let mut zones = vec![format!("{run}={}", plan.zone.zone_dir)];
    zones.extend(kit.map(|kit_dir| format!("{run}-kit={kit_dir}")));
    let zone_flags: String = zones.iter().map(|zone| format!(" --zone {zone}")).collect();
    let did = format!(
        "mr init --origin <the copy's branch> --integration integration --remote origin{zone_flags} --root-file for \
         each of {} top-level files --replace --actor accept",
        plan.root_files.len()
    );
    let branch = world.git(copy, &["symbolic-ref", "--short", "HEAD"]);
    let (held, saw) = if branch.code == Some(0) {
        let mut args = vec!["init", "--origin", branch.stdout.trim(), "--integration", "integration"];
        args.extend(["--remote", "origin"]);
        for zone in &zones {
            args.extend(["--zone", zone.as_str()]);
        }
        for file in &plan.root_files {
            args.extend(["--root-file", file.as_str()]);
        }
        args.push("--replace");
        let reply = actor_mr(world, copy, &args);
        (["written", "already-done", "replaced"].iter().any(|outcome| reply.is(0, outcome)), reply.saw)
    } else {
        (false, "the copy's HEAD is detached; check out a branch".to_string())
    };
    if kit.is_none() {
        return checks.check(&did, held, &saw);
    }
    checks.log(&format!("{did}: {saw}"));
    held
}
