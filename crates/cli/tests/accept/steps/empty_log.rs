//! Whether `mr log --json` reported an empty log.

use crate::json::{parse_json, JsonValue};

/// Ok only when the exit is 0, stdout is one JSON value, its outcome is "empty" and its records are an empty
/// array. The text is the check's "saw": the exit, the outcome and the record count (`exit 0; outcome empty; 0
/// records`), naming whichever of them is missing or of the wrong kind, or the parse error when stdout is not JSON.
pub fn empty_log(code: Option<i32>, stdout: &str) -> (bool, String) {
    let exit = code.map_or_else(|| "no exit code".to_string(), |code| format!("exit {code}"));
    let report = match parse_json(stdout) {
        Ok(report) => report,
        Err(error) => return (false, format!("{exit}; stdout is not JSON: {error}")),
    };
    let outcome = match report.field("outcome") {
        Some(JsonValue::Str(outcome)) => format!("outcome {outcome}"),
        Some(_) => "outcome is not a string".to_string(),
        None => "no outcome field".to_string(),
    };
    let records = match report.field("records") {
        Some(JsonValue::Array(records)) if records.len() == 1 => "1 record".to_string(),
        Some(JsonValue::Array(records)) => format!("{} records", records.len()),
        Some(_) => "records is not an array".to_string(),
        None => "no records field".to_string(),
    };
    let ok = code == Some(0)
        && report.field("outcome") == Some(&JsonValue::Str("empty".to_string()))
        && report.field("records") == Some(&JsonValue::Array(Vec::new()));
    (ok, format!("{exit}; {outcome}; {records}"))
}
