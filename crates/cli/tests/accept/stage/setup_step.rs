//! Records one set-up line of a step and says whether the set-up can go on.

use super::Reply;
use crate::report::Checks;

/// One set-up check line, did `did`, holding when `reply` exited 0 with `outcome`, saw the reply's `saw`. Err
/// `<did>: <saw>` when it did not hold, the text the steps' later checks name as why they did not run.
pub fn setup_step(checks: &mut Checks, did: &str, reply: &Reply, outcome: &str) -> Result<(), String> {
    if checks.check(did, reply.is(0, outcome), &reply.saw) {
        return Ok(());
    }
    Err(format!("{did}: {}", reply.saw))
}
