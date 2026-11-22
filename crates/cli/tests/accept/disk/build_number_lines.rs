//! The log lines for one zone's debug build in step 5.

use super::{byte_text, millis_text, ZoneNumbers};
use crate::report::Checks;

/// Log lines, never checks, so they never change the step's verdict. A build taken gives two: its wall time, and the
/// request folder made after it, with its build output; a build not taken gives one saying why, and the request
/// folder's bytes with build output are not shown, there being none.
pub fn build_number_lines(label: &str, numbers: &ZoneNumbers, checks: &mut Checks) {
    let wall = match &numbers.build {
        Ok(wall) => millis_text(*wall),
        Err(why) => return checks.log(&format!("number: {label}: the debug build: not taken: {why}")),
    };
    checks.log(&format!("number: {label}: the debug build's wall time: {wall}"));
    let built = match &numbers.built_request {
        Ok(bytes) => byte_text(*bytes),
        Err(why) => format!("not taken: {why}"),
    };
    checks.log(&format!(
        "number: {label}: the request folder made after the build (its files and build output, apparent bytes, .git \
         left out): {built}"
    ));
}
