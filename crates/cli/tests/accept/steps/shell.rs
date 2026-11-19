//! Step 0, the shell: proves the world, the `mr` runner and the kill path work, with `mr init` and `mr log`.
//! The log is empty after the two inits because init records no op: it writes config.json only and moves no ref
//! (crates/cli/src/init/run_init.rs writes only the config). Steps 3 and 4 check real records
//! (the op log, after the kills and for the agent's actor).

use std::path::Path;

use super::{empty_log, Gate, StepSpec};
use crate::report::Checks;
use crate::world::{Crash, World};

pub const SHELL: StepSpec =
    StepSpec { number: 0, title: "the shell", order: "ACCEPT-0", gate: Gate::Always, body: Some(check_shell) };

const INIT: &[&str] = &[
    "init", "--actor", "accept", "--origin", "main", "--integration", "integration", "--remote", "origin", "--zone",
    "a=src/a", "--zone", "b=src/b", "--root-file", "Cargo.toml", "--json",
];

/// `; exit $?` keeps sh from exec-ing a lone command, so git's parent is the killed shell, as the shim requires.
const UPDATE_REF: &str = "git -C \"$0\" update-ref refs/heads/accept-kill HEAD; exit $?";

fn check_shell(world: &World, checks: &mut Checks) {
    check_repositories(world, checks);
    for (did, outcome) in [("mr init in work", "written"), ("mr init again", "already-done")] {
        let ran = world.mortar(&world.work, INIT);
        let ok = ran.code == Some(0) && ran.stdout.contains(&format!("\"outcome\":\"{outcome}\""));
        checks.check(did, ok, &format!("{}; {}", ran.seen(), ran.stdout.trim()));
    }
    let log = world.mortar(&world.work, &["log", "--json"]);
    let (ok, saw) = empty_log(log.code, &log.stdout);
    checks.check("mr log --json in work after init", ok, &saw);
    for (crash, want) in [(Crash::Before, ""), (Crash::After, world.base.as_str())] {
        check_kill(world, checks, crash, want);
    }
}

/// work's main, origin's main, integration and the second clone's HEAD all sit at the one base commit.
fn check_repositories(world: &World, checks: &mut Checks) {
    let origin = world.origin.to_string_lossy();
    let seen = [
        ("main", rev(world, &world.work, &["rev-parse", "main"])),
        ("origin main", rev(world, &world.root, &["--git-dir", &origin, "rev-parse", "main"])),
        ("integration", rev(world, &world.work, &["rev-parse", "integration"])),
        ("other HEAD", rev(world, &world.other, &["rev-parse", "HEAD"])),
    ];
    let ok = seen.iter().all(|(_, value)| *value == world.base);
    let saw: Vec<String> = seen.iter().map(|(name, value)| format!("{name} {value}")).collect();
    checks.check(&format!("made the repositories at base {}", world.base), ok, &saw.join(", "));
}

/// A shell killed at its update-ref of refs/heads/accept-kill: signal 9, and the ref absent (before) or at base
/// (after).
fn check_kill(world: &World, checks: &mut Checks, crash: Crash, want: &str) {
    let did = format!("killed sh at update-ref, {}", crash.as_str());
    let work = world.work.to_string_lossy();
    match world.killed(&world.work, Path::new("sh"), &["-c", UPDATE_REF, &work], crash) {
        Err(error) => {
            checks.check(&did, false, &format!("the kill path failed: {error}"));
        }
        Ok(ran) => {
            let at = rev(world, &world.work, &["rev-parse", "-q", "--verify", "refs/heads/accept-kill"]);
            let shown = if at.is_empty() { "absent" } else { at.as_str() };
            checks.check(&did, ran.signal == Some(9) && at == want, &format!("{}; accept-kill {shown}", ran.seen()));
        }
    }
}

/// The trimmed stdout of a git read: empty when git found nothing.
fn rev(world: &World, cwd: &Path, args: &[&str]) -> String {
    world.git(cwd, args).stdout.trim().to_string()
}
