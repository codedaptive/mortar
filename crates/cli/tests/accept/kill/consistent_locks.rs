//! 03's "no lock left held": each Mortar lock file's recorded pid is not alive. Real files and processes:
//! kill_consistent.rs reads the locks after every case on every drive.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use super::{common_dir, lock_pid};
use crate::world::World;

/// Whether no lock is held, and what was seen. Each file under `<common>/mortar/locks` (lock files and their
/// `.takeover` guards) is held when its recorded pid (lock_pid) is alive, by `kill -0` through `sh`, or when it
/// records none; a file gone before it was read, or no locks folder, holds nothing. A SIGKILLed holder leaves its
/// file with a dead pid, which the next verb takes over, so a file is not by itself a held lock.
pub fn consistent_locks(world: &World) -> (bool, String) {
    let folder = common_dir(world).join("mortar").join("locks");
    let entries = match fs::read_dir(&folder) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => return (true, "no locks folder".to_string()),
        Err(error) => return (false, format!("{} could not be listed: {error}", folder.display())),
    };
    let mut held = false;
    let mut seen = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let text = match fs::read_to_string(entry.path()) {
            Ok(text) => text,
            Err(error) if error.kind() == ErrorKind::NotFound => continue,
            Err(error) => {
                held = true;
                seen.push(format!("{name}: unreadable: {error}"));
                continue;
            }
        };
        let Some(pid) = lock_pid(&text) else {
            held = true;
            seen.push(format!("{name}: no pid recorded"));
            continue;
        };
        let alive = pid_alive(world, pid);
        held |= alive;
        seen.push(format!("{name}: pid {pid} {}", if alive { "alive" } else { "not alive" }));
    }
    let saw = if seen.is_empty() { "the locks folder is empty".to_string() } else { seen.join(", ") };
    (!held, saw)
}

/// `kill -0 <pid>` exits 0: the process exists.
fn pid_alive(world: &World, pid: u32) -> bool {
    world
        .command(Path::new("sh"), &world.root)
        .args(["-c", "kill -0 \"$0\" 2>/dev/null", &pid.to_string()])
        .output()
        .is_ok_and(|output| output.status.success())
}
