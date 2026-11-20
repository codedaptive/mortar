//! An edit saved with `mr save`. Real files and processes: step 1 (flow/run_flow.rs) saves through it on every
//! drive.

use std::path::Path;

use super::{agent_mr, read_reply, stage_edit, Reply};
use crate::world::{Ran, World};

/// Writes `text` to `file` in `folder` (stage_edit), then `mr save "<sentence>" --json` there. An edit that cannot
/// be written runs no save; its reply names the error.
pub fn stage_save(world: &World, folder: &Path, file: &str, text: &str, sentence: &str) -> Reply {
    match stage_edit(folder, file, text) {
        Ok(()) => agent_mr(world, folder, &["save", sentence]),
        Err(error) => read_reply(&Ran::from_error(&format!("the edit of {file}"), error)),
    }
}
