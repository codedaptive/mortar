//! A lock record as lock file text: four `name=value` lines, pid, host, key, since.

use super::{LockError, LockRecord};

/// The record's file text. Refuses a host or key that is empty or would break the line format.
pub fn encode_lock_record(r: &LockRecord) -> Result<String, LockError> {
    check_field("host", &r.host)?;
    check_field("key", &r.key)?;
    Ok(format!(
        "pid={}\nhost={}\nkey={}\nsince={}\n",
        r.pid, r.host, r.key, r.since
    ))
}

fn check_field(name: &str, value: &str) -> Result<(), LockError> {
    if value.is_empty() {
        return Err(LockError::BadRecord(format!("{name} is empty")));
    }
    if value.contains(['\n', '\r']) {
        return Err(LockError::BadRecord(format!(
            "{name} {value:?} holds a line break"
        )));
    }
    Ok(())
}
