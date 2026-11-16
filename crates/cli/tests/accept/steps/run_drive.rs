//! The whole drive: every Done step, the report written and printed. Real repositories, processes and files: the
//! one case in main.rs, `done_list`, is this run.

use std::path::{Path, PathBuf};

use super::{run_step, DONE_LIST};
use crate::report::{drive_passed, format_report, report_path, write_report, Block};

/// Runs the Done list with the process environment, writes the report to report_path's choice (the workspace is
/// two folders above this crate) and prints it. Returns the report's path and whether the drive passed: no step
/// failed and the report was written.
pub fn run_drive() -> (PathBuf, bool) {
    let env = |var: &str| std::env::var_os(var);
    let blocks: Vec<Block> = DONE_LIST.iter().map(|spec| run_step(spec, &env)).collect();
    let text = format_report(&blocks);
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace = manifest.ancestors().nth(2).unwrap_or(manifest);
    let path = report_path(env("ACCEPT_REPORT").as_deref(), env("CARGO_TARGET_DIR").as_deref(), workspace);
    let written = write_report(&path, &text);
    println!("{text}");
    match &written {
        Ok(()) => println!("report written to {}", path.display()),
        Err(error) => println!("could not write the report to {}: {error}", path.display()),
    }
    (path, drive_passed(&blocks) && written.is_ok())
}
