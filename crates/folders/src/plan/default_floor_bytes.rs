//! The free-disk floor used when the caller sets none.

/// The spec's 10 GB, in decimal bytes (10 * 10^9), the unit disk sizes and `df -H` are stated in;
/// under it mortar refuses to make a folder and says what a sweep would free.
pub const DEFAULT_FLOOR_BYTES: u64 = 10_000_000_000;
