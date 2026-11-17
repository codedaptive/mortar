//! Step 2's checks 3 and 4: `git commit` on a plain checkout of a guarded branch. Real repositories and processes:
//! run_guard.rs runs it on every drive.

use std::path::PathBuf;

use super::{guard_refused, local_ref, refusal_parts};
use crate::report::Checks;
use crate::stage::stage_edit;
use crate::world::World;

/// Check `number`: a plain `git worktree add` checkout of `branch` at `<root>/checkout-<branch>`, a new file
/// guard.txt added there, then `git commit`, which the guard must refuse with `refs/heads/<branch>` unmoved
/// (guard_refused). When the checkout or the add fails, the check is a failed line saying so.
pub fn guard_commit(world: &World, checks: &mut Checks, number: u8, branch: &str) {
    let did = format!("{number}. git commit on a plain worktree of {branch}");
    let refname = format!("refs/heads/{branch}");
    let ready = checkout(world, checks, number, branch).and_then(|path| {
        stage_edit(&path, "guard.txt", "a commit the guard refuses\n").map_err(|error| format!("guard.txt: {error}"))?;
        let added = world.git(&path, &["add", "guard.txt"]);
        if added.code != Some(0) {
            return Err(format!("git add guard.txt: {}", added.seen()));
        }
        Ok(path)
    });
    match ready {
        Ok(path) => {
            let commit: &[&str] = &["commit", "-q", "-m", "a commit the guard refuses"];
            let why = format!("only Mortar moves {refname}");
            guard_refused(world, checks, &did, (&path, commit), &why, &|| local_ref(world, &refname));
        }
        Err(why) => {
            checks.check(&did, false, &format!("not run: {why}"));
        }
    }
}

/// `git worktree add --force <root>/checkout-<branch> <branch>` in the working repository; --force because work has
/// main checked out, which git otherwise will not check out twice. When git fails, a failed set-up line, then the
/// same with the hooks off (`-c core.hooksPath=<root>/no-hooks`, a folder that does not exist) on a line of its own,
/// so the commit can still be checked: the checkout is the set-up, the commit is the check.
fn checkout(world: &World, checks: &mut Checks, number: u8, branch: &str) -> Result<PathBuf, String> {
    let path = world.root.join(format!("checkout-{branch}"));
    let text = path.to_string_lossy().into_owned();
    let add = ["worktree", "add", "--force", text.as_str(), branch];
    let did = format!("set-up for {number}: git worktree add --force <root>/checkout-{branch} {branch}");
    if added(world, checks, &did, &add) {
        return Ok(path);
    }
    let hooks = format!("core.hooksPath={}", world.root.join("no-hooks").display());
    let mut retry = vec!["-c", hooks.as_str()];
    retry.extend(add);
    let did = format!("{did}, with the hooks off (after the failure)");
    if added(world, checks, &did, &retry) {
        return Ok(path);
    }
    Err(format!("no checkout of {branch} could be made"))
}

/// Runs `git <args>` in work as one set-up line: true when it exited 0. A failure shows the guard's why when its
/// hook refused (refusal_parts), since git's first stderr line is only "Preparing worktree".
fn added(world: &World, checks: &mut Checks, did: &str, args: &[&str]) -> bool {
    let ran = world.git(&world.work, args);
    let (why, _, _) = refusal_parts(&ran.stderr);
    let saw = if why.is_empty() { ran.seen() } else { format!("{}; the guard: {why}", ran.seen()) };
    checks.check(did, ran.code == Some(0), &saw)
}
