//! A request folder made with `mr folder request`. Real processes: step 1 (flow/run_flow.rs) makes r1 to r4
//! through it on every drive.

use std::path::PathBuf;

use super::{harness_mr, Reply};
use crate::world::World;

/// `mr folder request <zone> <request> <root>/<request> --paths <paths>... --actor accept --json` in the working
/// repository; the folder's path and the reply.
pub fn stage_request(world: &World, zone: &str, request: &str, paths: &[&str]) -> (PathBuf, Reply) {
    let folder = world.root.join(request);
    let text = folder.to_string_lossy().into_owned();
    let mut args = vec!["folder", "request", zone, request, text.as_str(), "--paths"];
    args.extend_from_slice(paths);
    let reply = harness_mr(world, &args);
    (folder, reply)
}
