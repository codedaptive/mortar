//! draw_temp_path: a fresh, randomly drawn temp file name beside the file it will replace.
//! No file I/O (it only bumps a process-wide counter); draw_temp_path_tests covers it.

use std::collections::hash_map::RandomState;
use std::ffi::OsString;
use std::hash::BuildHasher;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Counts draws in this process, so no two draws hash the same input.
static DRAWS: AtomicU64 = AtomicU64::new(0);

/// Returns `<file name of path>.mortar-tmp-<16 lowercase hex digits>` in `path`'s own folder.
///
/// The hex is a hash of a process-wide draw counter under `RandomState`'s keys, which std seeds from OS
/// randomness: another writer cannot readily predict the name and plant a symlink there first. The clock is not used
/// because its value is guessable and two draws in one tick would repeat; the counter makes every draw in this
/// process differ in input.
pub(crate) fn draw_temp_path(path: &Path) -> PathBuf {
    let draw = RandomState::new().hash_one(DRAWS.fetch_add(1, Ordering::Relaxed));
    let mut name = OsString::from(path.file_name().unwrap_or_default());
    name.push(format!(".mortar-tmp-{draw:016x}"));
    path.with_file_name(name)
}
