//! no_refresh_parameters: the GIT_CONFIG_PARAMETERS value that turns off `git diff`'s index stat refresh.

use std::ffi::{OsStr, OsString};

/// `base` followed by the entry `'diff.autoRefreshIndex'='false'`, in git's GIT_CONFIG_PARAMETERS form
/// (whitespace-separated single-quoted entries), one space between when `base` is non-empty. Git reads the entries in
/// order, so this last one wins over any earlier setting of the same key.
pub(crate) fn no_refresh_parameters(base: Option<&OsStr>) -> OsString {
    let mut value = base.map(OsStr::to_os_string).unwrap_or_default();
    if !value.is_empty() {
        value.push(" ");
    }
    value.push("'diff.autoRefreshIndex'='false'");
    value
}
