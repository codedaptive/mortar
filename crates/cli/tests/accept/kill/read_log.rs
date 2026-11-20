//! Reads the op log with `mr log --json`. Real processes: run_kill.rs reads it after every kill and every repeat on
//! every drive.

use super::{op_lines, OpLine};
use crate::stage::read_reply;
use crate::world::World;

/// The records of `mr log --json` in the working repository (op_lines); an error naming what the run said when it
/// did not exit 0 with outcome "listed" or "empty".
pub fn read_log(world: &World) -> Result<Vec<OpLine>, String> {
    let reply = read_reply(&world.mortar(&world.work, &["log", "--json"]));
    if reply.is(0, "listed") || reply.is(0, "empty") {
        return Ok(op_lines(&reply.report));
    }
    Err(format!("mr log --json: {}", reply.saw))
}
