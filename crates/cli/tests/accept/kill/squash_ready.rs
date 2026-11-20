//! A zone's squash, ready to land. Real repositories and processes: run_kill.rs makes case 3's two squashes through
//! it on every drive.

use crate::report::Checks;
use crate::stage::harness_mr;
use crate::world::World;

/// `mr squash <zone> --actor accept --json`: one set-up line, holding when it exits 0 "squashed" with a commit. The
/// squash's commit, or an error naming what was seen.
pub fn squash_ready(world: &World, checks: &mut Checks, zone: &str) -> Result<String, String> {
    let squash = harness_mr(world, &["squash", zone]);
    let commit = squash.text("commit");
    let did = format!("set-up: mr squash {zone} --actor accept");
    let saw = format!("{}; commit {commit}", squash.saw);
    if checks.check(&did, squash.is(0, "squashed") && !commit.is_empty(), &saw) {
        return Ok(commit);
    }
    Err(format!("{did}: {saw}"))
}
