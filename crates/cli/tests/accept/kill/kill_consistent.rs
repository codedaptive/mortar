//! 03's "Consistent" list, after a case. Real repositories and processes: every case of run_kill.rs runs it on every
//! drive.

use std::path::PathBuf;

use super::{consistent_folders, consistent_locks, open_intents, read_log, refs_changed, CaseLines};
use crate::report::Checks;
use crate::stage::{git_out, Reply};
use crate::world::World;

/// The five items, one line each: `git fsck` exits 0 printing nothing; no lock held (consistent_locks); every folder
/// in `folders` at its branch and clean (consistent_folders); no open intent in the op log; and `repeat`, the
/// case's command once more, exits 0 with every ref of the working repository where it was.
pub fn kill_consistent(
    world: &World,
    checks: &mut Checks,
    lines: &mut CaseLines,
    folders: &[PathBuf],
    repeat: &dyn Fn() -> Reply,
) {
    let fsck = world.git(&world.work, &["fsck", "--no-progress", "--no-dangling"]);
    let printed = format!("{}{}", fsck.stdout, fsck.stderr);
    let first = printed.lines().map(str::trim).find(|line| !line.is_empty());
    let saw = format!("{}; {}", fsck.seen(), first.unwrap_or("printed nothing"));
    lines.check(checks, fsck.code == Some(0) && first.is_none(), &saw);
    let (ok, saw) = consistent_locks(world);
    lines.check(checks, ok, &saw);
    let (ok, saw) = consistent_folders(world, folders);
    lines.check(checks, ok, &saw);
    match read_log(world) {
        Ok(log) => {
            let open: Vec<String> = open_intents(&log).iter().map(|op| op.shown()).collect();
            lines.check(checks, open.is_empty(), &format!("open intents [{}]", open.join("; ")));
        }
        Err(error) => {
            lines.check(checks, false, &error);
        }
    }
    let listing = ["for-each-ref", "--format=%(refname) %(objectname)"];
    let before = git_out(world, &world.work, &listing);
    let reply = repeat();
    let changed = refs_changed(&before, &git_out(world, &world.work, &listing));
    let saw = format!("{}; refs moved [{}]", reply.saw, changed.join(", "));
    lines.check(checks, reply.exit == Some(0) && changed.is_empty(), &saw);
}
