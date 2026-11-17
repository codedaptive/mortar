//! ACCEPT-1's set-up (01-flow.md, "Set-up"), shared by steps 1-4. Real repositories and processes: step 1
//! (flow/run_flow.rs) runs it on every drive.

use std::path::PathBuf;

use super::{harness_mr, Stage};
use crate::report::Checks;
use crate::world::World;

const INIT: &[&str] = &[
    "init", "--origin", "main", "--integration", "integration", "--remote", "origin", "--zone", "a=src/a", "--zone",
    "b=src/b", "--root-file", "Cargo.toml",
];

/// In the working repository: `mr init`, `mr guard install`, then the folders of zones a and b at `<root>/zone-a`
/// and `<root>/zone-b`, one check line each. None when init or a zone folder failed; a guard that did not install
/// fails its line and the set-up goes on, since the flow does not rely on it (step 2 tests it).
pub fn stage_new(world: &World, checks: &mut Checks) -> Option<Stage> {
    let init = harness_mr(world, INIT);
    let did = "set-up: mr init --actor accept --origin main --integration integration --remote origin --zone a=src/a \
               --zone b=src/b --root-file Cargo.toml";
    let initialised = checks.check(did, init.is(0, "written"), &init.saw);
    let guard = harness_mr(world, &["guard", "install"]);
    checks.check("set-up: mr guard install --actor accept", guard.is(0, "installed"), &guard.saw);
    let zone_a = zone_folder(world, checks, "a");
    let zone_b = zone_folder(world, checks, "b");
    initialised.then_some(())?;
    Some(Stage { zone_a: zone_a?, zone_b: zone_b? })
}

/// `mr folder zone <zone> <root>/zone-<zone>` as the spec writes it. Init makes no zone branch, so when that is
/// refused the line fails and the folder is made again with `--start main` (the zone branch cut from origin's
/// commit), on a line of its own, so the later checks can still run.
fn zone_folder(world: &World, checks: &mut Checks, zone: &str) -> Option<PathBuf> {
    let path = world.root.join(format!("zone-{zone}"));
    let text = path.to_string_lossy().into_owned();
    let made = harness_mr(world, &["folder", "zone", zone, &text]);
    let did = format!("set-up: mr folder zone {zone} <root>/zone-{zone} --actor accept");
    if checks.check(&did, made.is(0, "made"), &made.saw) {
        return Some(path);
    }
    let started = harness_mr(world, &["folder", "zone", zone, &text, "--start", "main"]);
    let did = format!("set-up: mr folder zone {zone} <root>/zone-{zone} --start main --actor accept (after the refusal)");
    checks.check(&did, started.is(0, "made"), &started.saw).then_some(path)
}
