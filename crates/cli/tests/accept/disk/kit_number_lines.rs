//! The log lines for step 5's one-kit zone.

use super::{build_number_lines, byte_text, millis_text, ZoneNumbers};
use crate::report::Checks;

const ZONE: &str = "the mr folder zone folder (its files, apparent bytes, .git and target left out)";
const REQUEST: &str = "the mr folder request folder (its files, apparent bytes, .git and target left out)";
const STORE: &str = "mr disk's object store, counted once";
const SNAPSHOT: &str = "mr snapshot's wall time in the request folder after a small edit";

/// Log lines, never checks, so they never change the step's verdict. No kit measured: one line saying why. Else,
/// labelled with the kit's folder: the zone folder's and the request folder's bytes, build_number_lines, then mr
/// disk's store and mr snapshot's wall time, each its value or `not taken: <why>`.
pub fn kit_number_lines(kit: &Result<(String, ZoneNumbers), String>, checks: &mut Checks) {
    let (kit_dir, numbers) = match kit {
        Ok(measured) => measured,
        Err(why) => return checks.log(&format!("number: the one-kit zone: not taken: {why}")),
    };
    let label = format!("the one-kit zone ({kit_dir})");
    number_line(checks, &label, ZONE, bytes(&numbers.zone_folder));
    number_line(checks, &label, REQUEST, bytes(&numbers.request_folder));
    build_number_lines(&label, numbers, checks);
    number_line(checks, &label, STORE, bytes(&numbers.store));
    number_line(checks, &label, SNAPSHOT, numbers.snapshot.as_ref().map(|wall| millis_text(*wall)));
}

fn bytes(number: &Result<u64, String>) -> Result<String, &String> {
    number.as_ref().map(|n| byte_text(*n))
}

fn number_line(checks: &mut Checks, label: &str, what: &str, taken: Result<String, &String>) {
    let shown = taken.unwrap_or_else(|why| format!("not taken: {why}"));
    checks.log(&format!("number: {label}: {what}: {shown}"));
}
