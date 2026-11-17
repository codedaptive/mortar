//! How a check line shows a ref read with plain git.

/// The commit, or "absent" when the read found none ("").
pub fn shown_ref(value: &str) -> &str {
    if value.is_empty() {
        "absent"
    } else {
        value
    }
}
