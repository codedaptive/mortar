//! random_token: A token for a temp file's name that is unlikely to repeat, so a link is unlikely to be planted at
//! the name ahead of time. Not a secret: the temp file's exclusive creation is what guarantees no link is followed.
//! No sibling test: it reads process randomness. Install passes it to apply_step (through apply_install) and
//! write_record (through install_in); the install integration tests that run install_in reach it that way, while temp_files.rs
//! passes fixed tokens instead, to collide with a planted name on purpose.

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Counts this process's calls, so two calls differ even when the clock and the hasher's keys repeat.
static CALLS: AtomicU64 = AtomicU64::new(0);

/// A number that is unlikely to repeat between processes or between calls in one process: the process id, the
/// clock's nanoseconds and a call counter, hashed with RandomState's keys (random per process).
pub fn random_token() -> u64 {
    let mut hasher = RandomState::new().build_hasher();
    hasher.write_u32(process::id());
    // A clock before 1970 gives 0 here; the counter and the hasher's keys still vary the token.
    hasher.write_u128(SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |since| since.as_nanos()));
    hasher.write_u64(CALLS.fetch_add(1, Ordering::Relaxed));
    hasher.finish()
}
