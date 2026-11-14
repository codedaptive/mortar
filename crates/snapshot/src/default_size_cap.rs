//! DEFAULT_SIZE_CAP: the size cap a caller passes when it has none of its own. A constant only; cap_refusals_tests
//! exercises it through the files it refuses and keeps.

/// In bytes: 5 MiB. A new file larger than this is left out of a snapshot.
pub const DEFAULT_SIZE_CAP: u64 = 5 * 1024 * 1024;
