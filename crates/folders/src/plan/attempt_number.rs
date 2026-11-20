//! The rule for an attempt number written as text, shared by attempt branch names and folder records.

/// `text` as an attempt number: ASCII digits only, so u32's parse cannot take a sign or space; not starting with '0',
/// so each number has one spelling and 0 is out; fitting u32. None otherwise, the empty text included.
pub(crate) fn attempt_number(text: &str) -> Option<u32> {
    if text.starts_with('0') || !text.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}
