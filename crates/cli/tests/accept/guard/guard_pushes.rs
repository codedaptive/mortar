//! Step 2's checks 5-7: plain pushes of guarded branches from the working repository. Real repositories and
//! processes: run_guard.rs runs it on every drive.

use super::guard_refused;
use crate::report::Checks;
use crate::stage::{git_out, remote_git};
use crate::world::World;

/// Check 5: `git push origin <other>:refs/heads/main`. Check 6: the same push by the exact text of
/// `git remote get-url --push origin`, since the guard knows a push by URL only when the text matches a push URL
/// (S-100). Check 7: `git push origin <other>:refs/heads/integration`, applicable because the guard guards
/// integration on push as well. Each must be refused with the bare remote's branch unmoved (guard_refused).
pub fn guard_pushes(world: &World, checks: &mut Checks, other: &str) {
    let url = git_out(world, &world.work, &["remote", "get-url", "--push", "origin"]);
    checks.log(&format!("origin's push URL, from git remote get-url --push origin: {url}"));
    let pushes = [
        ("5. git push origin <other commit>:refs/heads/main, from work", "origin", "main"),
        ("6. git push <origin's push URL> <other commit>:refs/heads/main, from work", url.as_str(), "main"),
        ("7. git push origin <other commit>:refs/heads/integration, from work", "origin", "integration"),
    ];
    for (did, to, branch) in pushes {
        let refname = format!("refs/heads/{branch}");
        let spec = format!("{other}:{refname}");
        let why = format!("only Mortar pushes {refname} to origin");
        let read = || remote_git(world, &["rev-parse", "-q", "--verify", &refname]);
        guard_refused(world, checks, did, (&world.work, &["push", to, spec.as_str()]), &why, &read);
    }
}
