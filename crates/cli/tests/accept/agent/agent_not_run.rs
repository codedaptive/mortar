//! The failed lines for step 4's checks when the set-up left them impossible.

use super::AGENT_CHECKS;
use crate::report::Checks;

/// One failed line for each of AGENT_CHECKS, saw `not run: <why>`, so the block always lists them all.
pub fn agent_not_run(checks: &mut Checks, why: &str) {
    for title in AGENT_CHECKS {
        checks.check(title, false, &format!("not run: {why}"));
    }
}
