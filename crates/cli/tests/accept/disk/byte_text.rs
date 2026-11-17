//! Shows a byte count on a check line.

/// "<bytes> bytes (<MiB to one decimal> MiB)", a MiB being 1048576 bytes.
pub fn byte_text(bytes: u64) -> String {
    // No From<u64> for f64 exists (it can lose precision); one decimal of MiB needs far less than f64 keeps.
    format!("{bytes} bytes ({:.1} MiB)", bytes as f64 / 1_048_576.0)
}
