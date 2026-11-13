//! A ref's value from the answer of `git rev-parse -q --verify <ref>`.

use gitio::Oid;

use super::OplogError;

/// Exit 1: the ref is absent (None). Exit 0: the one line git printed, as an object id. Any other exit,
/// or a line that is not an object id, is Malformed naming what git answered.
pub fn ref_value_from_output(status: i32, stdout: &[u8]) -> Result<Option<Oid>, OplogError> {
    let text = String::from_utf8_lossy(stdout);
    match status {
        1 => Ok(None),
        0 => {
            let line = text.strip_suffix('\n').unwrap_or(&text);
            Oid::parse(line).map(Some).map_err(|_| {
                malformed(format!("git rev-parse printed {line:?}, which is not a git object id"))
            })
        }
        other => Err(malformed(format!(
            "git rev-parse exited {other} (expected 0 or 1) and printed {text:?}"
        ))),
    }
}

fn malformed(reason: String) -> OplogError {
    OplogError::Malformed { line: None, reason }
}
