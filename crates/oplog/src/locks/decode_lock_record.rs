//! Lock file text back to a record: the strict inverse of encode_lock_record.

use std::str::FromStr;

use super::{LockError, LockRecord};

/// The record in `text`, which must be exactly the four lines encode_lock_record writes.
pub fn decode_lock_record(text: &str) -> Result<LockRecord, LockError> {
    let body = text
        .strip_suffix('\n')
        .ok_or_else(|| malformed("does not end in a newline".to_string()))?;
    let lines: Vec<&str> = body.split('\n').collect();
    let [pid, host, key, since] = lines.as_slice() else {
        return Err(malformed(format!("has {} lines, not 4", lines.len())));
    };
    Ok(LockRecord {
        pid: number(field("pid", pid)?, "pid")?,
        host: text_value(field("host", host)?, "host")?,
        key: text_value(field("key", key)?, "key")?,
        since: number(field("since", since)?, "since")?,
    })
}

fn malformed(reason: String) -> LockError {
    LockError::Malformed {
        path: None,
        reason,
        removed_guards: Vec::new(),
    }
}

/// The value of `line`, which must be `<name>=<value>`; split at the first '='.
fn field<'a>(name: &str, line: &'a str) -> Result<&'a str, LockError> {
    match line.split_once('=') {
        Some((found, value)) if found == name => Ok(value),
        Some((found, _)) => Err(malformed(format!("expected field {name}, found {found:?}"))),
        None => Err(malformed(format!("line {line:?} has no '='"))),
    }
}

/// Plain decimal digits only, no leading zero: what encode writes, so decode never accepts a spelling
/// encode would not produce.
fn number<T: FromStr>(value: &str, name: &str) -> Result<T, LockError> {
    let canonical = !value.is_empty()
        && value.bytes().all(|b| b.is_ascii_digit())
        && (value == "0" || !value.starts_with('0'));
    canonical
        .then(|| value.parse().ok())
        .flatten()
        .ok_or_else(|| malformed(format!("{name} {value:?} is not a number in range")))
}

fn text_value(value: &str, name: &str) -> Result<String, LockError> {
    if value.is_empty() {
        return Err(malformed(format!("{name} is empty")));
    }
    if value.contains('\r') {
        return Err(malformed(format!("{name} {value:?} holds a line break")));
    }
    Ok(value.to_string())
}
