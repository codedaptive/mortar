//! One refusal check of step 2 (checks 1-7): a plain git command the guard must refuse. Real repositories and
//! processes: run_guard.rs, guard_commit.rs and guard_pushes.rs run it on every drive.

use std::path::Path;

use super::{refusal_parts, refusal_ok, shown_ref};
use crate::report::Checks;
use crate::world::World;

/// Reads the ref the command must not move with `read` ("" when it is absent), runs `git <args>` in `cwd`, reads
/// the ref again, and writes one check line `did`: it holds when git exited non-zero, the ref did not move, and
/// git's output gives the guard's refusal with why `why` and the guard's owner and next (refusal_ok). The output is
/// stdout then stderr: git passes a reference-transaction hook's output to its stderr but a pre-push hook's to its
/// stdout. The line shows the exit, the ref before and after, and the three parts read.
pub fn guard_refused(
    world: &World,
    checks: &mut Checks,
    did: &str,
    (cwd, args): (&Path, &[&str]),
    why: &str,
    read: &dyn Fn() -> String,
) {
    let before = read();
    let ran = world.git(cwd, args);
    let after = read();
    let failed = matches!(ran.code, Some(code) if code != 0);
    let output = format!("{}{}", ran.stdout, ran.stderr);
    let (seen_why, owner, next) = refusal_parts(&output);
    let ok = failed && before == after && refusal_ok(&output, why);
    let saw = format!(
        "{}; ref before {}, after {}; read from git's stdout and stderr: why {seen_why:?}; owner {owner:?}; next \
         {next:?}",
        ran.seen(),
        shown_ref(&before),
        shown_ref(&after)
    );
    checks.check(did, ok, &saw);
}
