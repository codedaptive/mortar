//! The folder of codeteam's team manifests, from the value of ACCEPT_TEAMS. Pure: the caller reads the variable.

use std::ffi::OsString;
use std::path::PathBuf;

/// The folder `value` names. Err naming ACCEPT_TEAMS when it is unset or empty: the drive has no default folder,
/// so step 5's refusal fails closed rather than read the manifests from a guessed place.
pub fn teams_folder(value: Option<OsString>) -> Result<PathBuf, String> {
    value
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| "ACCEPT_TEAMS is not set: set it to the folder of codeteam's team manifests".to_string())
}
