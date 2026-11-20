//! The check lines for step 5's numbers.

use super::{byte_text, millis_text, DiskNumbers};
use crate::report::Checks;

/// Five check lines in a fixed order, one per number: ok and its text when taken, failed with `not taken: <why>`
/// otherwise, so the step passes only when every number was taken.
pub fn disk_number_lines(numbers: &DiskNumbers, checks: &mut Checks) {
    number_line(checks, "number: a full checkout of the copy (HEAD's files, apparent bytes)", bytes(&numbers.checkout));
    number_line(
        checks,
        "number: the mr folder zone folder (its files, apparent bytes, .git and target left out)",
        bytes(&numbers.zone_folder),
    );
    number_line(
        checks,
        "number: the mr folder request folder (its files, apparent bytes, .git and target left out)",
        bytes(&numbers.request_folder),
    );
    number_line(checks, "number: mr disk's object store, counted once", bytes(&numbers.store));
    number_line(
        checks,
        "number: mr snapshot's wall time in the request folder after a small edit",
        numbers.snapshot.as_ref().map(|wall| millis_text(*wall)),
    );
}

fn bytes(number: &Result<u64, String>) -> Result<String, &String> {
    number.as_ref().map(|n| byte_text(*n))
}

fn number_line(checks: &mut Checks, did: &str, taken: Result<String, &String>) {
    match taken {
        Ok(saw) => checks.check(did, true, &saw),
        Err(why) => checks.check(did, false, &format!("not taken: {why}")),
    };
}
