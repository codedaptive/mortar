//! with_object_quarantine: runs a closure with a git whose new objects go to a throwaway directory.
//! Why: the joined answer disk reports comes from saved_state, which builds the folder's tree with git add and
//! write-tree, and those write blobs and trees. With GIT_OBJECT_DIRECTORY set to a fresh directory in the OS temp
//! dir, the writes land there, and GIT_ALTERNATE_OBJECT_DIRECTORIES names the repository's own objects so every
//! read still finds them. The directory is removed when the closure returns, fails or panics, so disk writes
//! nothing in the repository and leaves nothing behind.

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use gitio::Git;

/// Counts the quarantines this process has made, so two in the same nanosecond still get their own directory.
static MADE: AtomicU64 = AtomicU64::new(0);

/// `act`'s answer, run with a clone of `git` whose env adds GIT_OBJECT_DIRECTORY=<a new directory under the OS temp
/// dir> and GIT_ALTERNATE_OBJECT_DIRECTORIES=`objects` (which must be absolute: git reads it from every folder).
/// The directory is named for the process id, a per-process count and the time, and made with create_dir, so an
/// existing one is never reused: making it is the only Err, with the reason. It is removed when this returns, and
/// when `act` panics.
pub fn with_object_quarantine<T>(git: &Git, objects: &Path, act: impl FnOnce(&Git) -> T) -> Result<T, String> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|since| since.as_nanos()).unwrap_or_default();
    let dir = std::env::temp_dir().join(format!(
        "mortar-disk-objects-{}-{}-{nanos}",
        process::id(),
        MADE.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&dir)
        .map_err(|error| format!("the object quarantine {} could not be made: {error}", dir.display()))?;
    let quarantine = RemovedOnDrop(dir);
    let mut quarantined = git.clone();
    quarantined.env.push((OsString::from("GIT_OBJECT_DIRECTORY"), quarantine.0.clone().into_os_string()));
    quarantined.env.push((OsString::from("GIT_ALTERNATE_OBJECT_DIRECTORIES"), objects.as_os_str().to_owned()));
    Ok(act(&quarantined))
}

/// The quarantine directory, removed with everything in it when dropped. A removal that fails leaves it in the
/// OS temp dir, outside the repository.
struct RemovedOnDrop(PathBuf);

impl Drop for RemovedOnDrop {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
