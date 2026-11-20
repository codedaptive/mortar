//! This host's name from the output of `hostname`: one word, no spaces or control characters.

use super::LockError;

/// The trimmed host name, or HostUnknown when the output is not one clean word.
pub(crate) fn parse_host_name(stdout: &[u8]) -> Result<String, LockError> {
    let text = std::str::from_utf8(stdout)
        .map_err(|_| LockError::HostUnknown("hostname output is not UTF-8".to_string()))?;
    let name = text.trim();
    if name.is_empty() {
        return Err(LockError::HostUnknown("hostname printed nothing".to_string()));
    }
    if name.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err(LockError::HostUnknown(format!(
            "hostname printed {name:?}, which holds whitespace or a control character"
        )));
    }
    Ok(name.to_string())
}
