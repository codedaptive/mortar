//! Step 2, the guard. Real repositories and processes: the drive (steps/guard.rs) runs it as step 2's
//! body on every drive.

use super::{guard_commit, guard_land, guard_pushes, guard_refused, guard_scratch, guard_squash, local_ref, other_commit};
use crate::report::Checks;
use crate::stage::stage_new;
use crate::world::World;

/// The set-up: stage_new (step 1's set-up through `mr guard install`), then a squash ready to land
/// (guard_squash). Then the nine checks in order, each at least one line: 1-2 `git update-ref` of integration and
/// main, 3-4 a commit on a checkout of each (guard_commit), 5-7 pushes (guard_pushes), 8 Mortar's own land and
/// publish (guard_land), 9 an unguarded ref (guard_scratch). Checks 1-7 and 9 need no folder, so they run whatever
/// the set-up did; check 8 needs the squash and is a failed line naming why when there is none.
pub fn run_guard(world: &World, checks: &mut Checks) {
    let squash = match stage_new(world, checks) {
        Some(_) => guard_squash(world, checks),
        None => Err("the set-up (init, guard install, zone folders) failed: see its lines".to_string()),
    };
    let other = other_commit(world);
    checks.log(&format!("the other commit, made on base {} with git commit-tree: {other}", world.base));
    for (number, branch) in [(1, "integration"), (2, "main")] {
        let refname = format!("refs/heads/{branch}");
        let did = format!("{number}. git update-ref {refname} <other commit>");
        let why = format!("only Mortar moves {refname}");
        let args = ["update-ref", refname.as_str(), other.as_str()];
        guard_refused(world, checks, &did, (&world.work, &args), &why, &|| local_ref(world, &refname));
    }
    guard_commit(world, checks, 3, "integration");
    guard_commit(world, checks, 4, "main");
    guard_pushes(world, checks, &other);
    guard_land(world, checks, squash);
    guard_scratch(world, checks, &other);
}
