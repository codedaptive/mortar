//! The failed lines for the checks an earlier failure left impossible.

use super::FLOW_CHECKS;
use crate::report::Checks;

/// One failed line for each check from `from` (1-based) to 15, did `<n>. <title>`, saw `not run: <why>`, so the
/// block always lists all fifteen.
pub fn not_run(checks: &mut Checks, from: usize, why: &str) {
    for (index, title) in FLOW_CHECKS.iter().enumerate().skip(from.saturating_sub(1)) {
        checks.check(&format!("{}. {title}", index + 1), false, &format!("not run: {why}"));
    }
}
