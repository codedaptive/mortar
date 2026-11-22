//! `mr snapshot`'s wall time after a small edit. Real files, processes and a clock: the drive (steps/disk.rs) runs
//! it through measure_zone in a zone's request folder, for each zone (the all-packages one, and the one-kit one when
//! the plan has a kit) whose two folders were made.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::time::{Duration, Instant};

use super::actor_mr;
use crate::world::World;

/// Appends "accept disk: a small edit\n" to `<folder>/<edit_file>`, then times one `mr snapshot --actor accept
/// --json` in `folder`, the clock around that run alone. Ok(wall) when it exits 0 with outcome "snapshotted"; else
/// Err with the reply's saw, or naming the file the edit could not be written to.
pub fn time_snapshot(world: &World, folder: &Path, edit_file: &str) -> Result<Duration, String> {
    let path = folder.join(edit_file);
    OpenOptions::new()
        .append(true)
        .open(&path)
        .and_then(|mut file| file.write_all(b"accept disk: a small edit\n"))
        .map_err(|error| format!("cannot append the small edit to {}: {error}", path.display()))?;
    let start = Instant::now();
    let reply = actor_mr(world, folder, &["snapshot"]);
    let wall = start.elapsed();
    if reply.is(0, "snapshotted") {
        Ok(wall)
    } else {
        Err(reply.saw)
    }
}
