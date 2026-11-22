//! A zone folder's debug build, timed. Real processes and a clock: the drive (steps/disk.rs) runs it through
//! measure_zone in each zone folder that was made and whose build was planned.

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use super::BuildCommand;
use crate::world::Ran;

/// Runs `program args` in `folder/dir`, stdin closed and output captured, with the drive's own environment (not the
/// world's, whose HOME at the scratch would hide the toolchain's caches) less every variable named CARGO_* but
/// CARGO_HOME, so the build queue's CARGO_TARGET_DIR cannot send the build output out of the folder. The clock runs
/// around that run alone. Ok(wall) on exit 0; else Err naming the command, how it ended and, when stderr has one,
/// its first line starting "error".
pub fn time_build(folder: &Path, build: &BuildCommand) -> Result<Duration, String> {
    let shown = format!("{} {} in {}", build.program, build.args.join(" "), folder.join(&build.dir).display());
    let mut command = Command::new(&build.program);
    command
        .args(&build.args)
        .current_dir(folder.join(&build.dir))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (name, _) in std::env::vars_os() {
        if name.to_str().is_some_and(|name| name.starts_with("CARGO_") && name != "CARGO_HOME") {
            command.env_remove(&name);
        }
    }
    let start = Instant::now();
    let output = command.output();
    let wall = start.elapsed();
    let ran = output.map_or_else(|error| Ran::from_error(&shown, error), Ran::from_output);
    if ran.code == Some(0) {
        return Ok(wall);
    }
    // seen() shows the first stderr line, which for cargo is a progress line; the first "error" line says why.
    match ran.stderr.lines().map(str::trim).find(|line| line.starts_with("error")) {
        Some(error) => Err(format!("{shown}: {} ({error})", ran.seen())),
        None => Err(format!("{shown}: {}", ran.seen())),
    }
}
