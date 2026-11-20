//! The rule for a zone or request name, which becomes one part of a branch name.

use super::PlanError;

/// Longest name accepted, in bytes: keeps branch names and the folder names made from them short.
const MAX_NAME_BYTES: usize = 100;

/// Refuses a name that is empty, over 100 bytes, holds anything but ASCII letters, digits, '-', '_'
/// or '.', starts with '.' or '-', holds "..", or ends with ".lock" or '.'. These keep the name one
/// valid git ref component that cannot be read as an option. The refusal names `what` and the name.
pub(crate) fn check_name(what: &str, name: &str) -> Result<(), PlanError> {
    match name_fault(name) {
        None => Ok(()),
        Some(fault) => Err(PlanError {
            reason: format!("{what} name {name:?} {fault}"),
        }),
    }
}

fn name_fault(name: &str) -> Option<&'static str> {
    if name.is_empty() {
        return Some("is empty");
    }
    if name.len() > MAX_NAME_BYTES {
        return Some("is over 100 bytes");
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    {
        return Some("holds a character other than ASCII letters, digits, '-', '_' or '.'");
    }
    if name.starts_with('.') || name.starts_with('-') {
        return Some("starts with '.' or '-'");
    }
    if name.contains("..") {
        return Some("holds \"..\"");
    }
    if name.ends_with(".lock") || name.ends_with('.') {
        return Some("ends with \".lock\" or '.'");
    }
    None
}
