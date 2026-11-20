//! An edit joined with `mr join`, which saves it first. Real files and processes: step 1 (flow/run_flow.rs) joins
//! through it on every drive.

use std::path::Path;

use super::{agent_mr, read_reply, stage_edit, Reply};
use crate::world::{Ran, World};

/// Writes `text` to `file` in `folder` (stage_edit), then `mr join --json` there; join saves the edit before it
/// joins. An edit that cannot be written runs no join; its reply names the error.
pub fn stage_join(world: &World, folder: &Path, file: &str, text: &str) -> Reply {
    match stage_edit(folder, file, text) {
        Ok(()) => agent_mr(world, folder, &["join"]),
        Err(error) => read_reply(&Ran::from_error(&format!("the edit of {file}"), error)),
    }
}
