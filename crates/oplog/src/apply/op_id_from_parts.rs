//! The text of an op id.

/// `<unix_nanos>-<pid>-<counter>`, each in lowercase hex. Hex holds no '-', so distinct parts give
/// distinct ids.
pub(crate) fn op_id_from_parts(unix_nanos: u128, pid: u32, counter: u64) -> String {
    format!("{unix_nanos:x}-{pid:x}-{counter:x}")
}
