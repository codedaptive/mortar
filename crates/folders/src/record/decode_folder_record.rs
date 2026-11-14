//! A folder record's JSON text back to the record.

use gitio::Oid;

use crate::plan::attempt_number;

use super::{parse_record_json, FolderKind, FolderRecord, JsonError, RecordError, RecordJsonValue};

/// The keys of a record, in the order encode_folder_record writes them; "attempt" is present on attempt records
/// only.
const KEYS: [&str; 8] = ["kind", "attempt", "zone", "request", "base", "work_order_paths", "needed_paths", "parent"];

/// The record in `text`, the strict inverse of encode_folder_record: one JSON object with exactly its keys,
/// each once, in any order, whitespace allowed around tokens. "attempt" is required on an attempt record and
/// refused on any other; its value is a decimal string from 1 to u32::MAX with no leading zero. A zone record's
/// request must be null, and a request or attempt record's must not be. Every refusal names the key, value or byte
/// that is wrong; text that stops before the record is complete says so.
pub fn decode_folder_record(text: &str) -> Result<FolderRecord, RecordError> {
    record(text).map_err(|reason| RecordError { reason })
}

fn record(text: &str) -> Result<FolderRecord, String> {
    let members = parse_record_json(text).map_err(|e| match e {
        JsonError::EndOfInput => "the text ends before the record is complete".to_string(),
        JsonError::Invalid(reason) => reason,
    })?;
    let [kind, attempt, zone, request, base, work_order_paths, needed_paths, parent] = fields(members)?;
    let kind = kind_named(take_string(kind, "kind")?, attempt)?;
    let request = take_optional_string(request, "request")?;
    match (kind, &request) {
        (FolderKind::Zone, Some(name)) => {
            return Err(format!("a zone record has request {name:?}; it must be null"))
        }
        (FolderKind::Request, None) => {
            return Err("a request record has request null; it must name the request".to_string())
        }
        (FolderKind::Attempt(_), None) => {
            return Err("an attempt record has request null; it must name the request".to_string())
        }
        _ => {}
    }
    let base = take_string(base, "base")?;
    Ok(FolderRecord {
        kind,
        zone: take_string(zone, "zone")?,
        request,
        base: Oid::parse(&base).map_err(|_| format!("base {base:?} is not a git object id"))?,
        work_order_paths: take_strings(work_order_paths, "work_order_paths")?,
        needed_paths: take_strings(needed_paths, "needed_paths")?,
        parent: take_optional_string(parent, "parent")?,
    })
}

/// One slot per name in KEYS, holding that key's value or None when absent; refuses an unknown key and a
/// key given twice.
fn fields(members: Vec<(String, RecordJsonValue)>) -> Result<[Option<RecordJsonValue>; 8], String> {
    let mut slots: [Option<RecordJsonValue>; 8] = Default::default();
    for (name, value) in members {
        let Some(i) = KEYS.iter().position(|k| *k == name) else {
            return Err(format!("unknown key {name:?}"));
        };
        if slots[i].replace(value).is_some() {
            return Err(format!("key {name:?} appears twice"));
        }
    }
    Ok(slots)
}

/// The kind `name` names; `attempt` is the "attempt" key's value, which only an attempt record may have.
fn kind_named(name: String, attempt: Option<RecordJsonValue>) -> Result<FolderKind, String> {
    match (name.as_str(), attempt) {
        ("attempt", attempt) => {
            let text = take_string(attempt, "attempt")?;
            attempt_number(&text).map(FolderKind::Attempt).ok_or_else(|| {
                format!("attempt {text:?} is not a decimal number from 1 to 4294967295 with no leading zero")
            })
        }
        ("zone" | "request", Some(_)) => Err(format!("\"attempt\" key on a {name} record")),
        ("zone", None) => Ok(FolderKind::Zone),
        ("request", None) => Ok(FolderKind::Request),
        _ => Err(format!("kind {name:?} is none of \"zone\", \"request\" or \"attempt\"")),
    }
}

fn take_string(slot: Option<RecordJsonValue>, name: &str) -> Result<String, String> {
    match slot {
        Some(RecordJsonValue::String(s)) => Ok(s),
        Some(_) => Err(format!("{name} is not a string")),
        None => Err(format!("missing key {name:?}")),
    }
}

fn take_optional_string(slot: Option<RecordJsonValue>, name: &str) -> Result<Option<String>, String> {
    match slot {
        Some(RecordJsonValue::String(s)) => Ok(Some(s)),
        Some(RecordJsonValue::Null) => Ok(None),
        Some(RecordJsonValue::Strings(_)) => Err(format!("{name} is not a string or null")),
        None => Err(format!("missing key {name:?}")),
    }
}

fn take_strings(slot: Option<RecordJsonValue>, name: &str) -> Result<Vec<String>, String> {
    match slot {
        Some(RecordJsonValue::Strings(items)) => Ok(items),
        Some(_) => Err(format!("{name} is not an array of strings")),
        None => Err(format!("missing key {name:?}")),
    }
}
