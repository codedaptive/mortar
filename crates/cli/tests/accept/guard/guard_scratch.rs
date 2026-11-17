//! Step 2's check 9: a ref the guard does not guard moves freely. Real repositories and processes: run_guard.rs
//! runs it on every drive.

use super::{local_ref, shown_ref};
use crate::report::Checks;
use crate::world::World;

/// `git update-ref refs/heads/scratch <other>` in the working repository: exit 0 and scratch at `other`.
pub fn guard_scratch(world: &World, checks: &mut Checks, other: &str) {
    let ran = world.git(&world.work, &["update-ref", "refs/heads/scratch", other]);
    let scratch = local_ref(world, "refs/heads/scratch");
    let ok = ran.code == Some(0) && !other.is_empty() && scratch == other;
    let saw = format!("{}; scratch {}", ran.seen(), shown_ref(&scratch));
    checks.check("9. git update-ref refs/heads/scratch <other commit>", ok, &saw);
}
