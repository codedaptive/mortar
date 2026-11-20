//! Which folders have been idle long enough to sweep.

use std::path::PathBuf;

use super::FolderAge;

const SECONDS_PER_DAY: u64 = 86_400;

/// The paths of `ages`, in input order, idle for at least `idle_days` at `now` (unix seconds): a folder with no
/// newest time is idle; one whose newest time is at or before `now` is idle when `now` minus it is at least
/// `idle_days` days (the day count saturates rather than overflows). A newest time after `now` is not idle.
pub fn idle_folders(ages: &[FolderAge], now: u64, idle_days: u64) -> Vec<PathBuf> {
    let idle_seconds = idle_days.saturating_mul(SECONDS_PER_DAY);
    ages.iter()
        .filter(|age| match age.newest {
            None => true,
            Some(t) => t <= now && now - t >= idle_seconds,
        })
        .map(|age| age.path.clone())
        .collect()
}
