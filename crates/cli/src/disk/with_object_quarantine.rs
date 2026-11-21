//! with_object_quarantine: runs a closure with a git whose new objects go to a throwaway directory.
//! Why: the joined answer disk reports comes from saved_state, which builds the folder's tree with git add and
//! write-tree, and those write blobs and trees. With GIT_OBJECT_DIRECTORY set to a fresh directory in the OS temp
//! dir, the writes land there, and GIT_ALTERNATE_OBJECT_DIRECTORIES names the repository's own objects so every
//! read still finds them. The directory is removed when the closure returns, fails or panics, so disk writes
//! nothing in the repository and leaves nothing behind.
//! The making and removing are passed into run_quarantined, so with_object_quarantine_tests proves without a real
//! directory that the remover runs after act returns and after act panics, that a failed make runs neither, and
//! how the name is drawn; quarantined_git_tests covers the env it builds. The integration test
//! crates/cli/tests/status_need_runs/disk_quarantine.rs::disk_leaves_no_object_quarantine_in_the_temp_dir
//! runs mr disk with its own empty TMPDIR, checks the joined answer it reports, and finds the TMPDIR
//! empty after. A reused name is impossible because create_dir refuses an existing directory.

use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use gitio::Git;

use super::quarantined_git;

/// Counts the quarantines this process has made, so two in the same nanosecond still get their own directory.
static MADE: AtomicU64 = AtomicU64::new(0);

/// `act`'s answer, run with a clone of `git` whose env adds GIT_OBJECT_DIRECTORY=<a new directory under the OS temp
/// dir> and GIT_ALTERNATE_OBJECT_DIRECTORIES=`objects` (which must be absolute: git reads it from every folder).
/// The directory is named for the process id, a per-process count and the time, and made with create_dir, so an
/// existing one is never reused: making it is the only Err, with the reason. It is removed when this returns, and
/// when `act` panics.
pub fn with_object_quarantine<T>(git: &Git, objects: &Path, act: impl FnOnce(&Git) -> T) -> Result<T, String> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|since| since.as_nanos()).unwrap_or_default();
    let dir = quarantine_dir(&std::env::temp_dir(), process::id(), MADE.fetch_add(1, Ordering::Relaxed), nanos);
    run_quarantined(
        git,
        objects,
        dir,
        |dir| fs::create_dir(dir),
        |dir| {
            let _ = fs::remove_dir_all(dir);
        },
        act,
    )
}

/// The quarantine directory's path under `temp`, named for the process id, the per-process count and the time.
fn quarantine_dir(temp: &Path, pid: u32, made: u64, nanos: u128) -> PathBuf {
    temp.join(format!("mortar-disk-objects-{pid}-{made}-{nanos}"))
}

/// `act`'s answer, run with `git` quarantined to `dir`: `make` makes `dir` first, and its failure is the only Err,
/// with neither `act` nor `remove` run; once it is made, `remove` is called on `dir` when this returns or `act`
/// panics.
fn run_quarantined<T>(
    git: &Git,
    objects: &Path,
    dir: PathBuf,
    make: impl FnOnce(&Path) -> std::io::Result<()>,
    remove: impl FnMut(&Path),
    act: impl FnOnce(&Git) -> T,
) -> Result<T, String> {
    make(&dir).map_err(|error| format!("the object quarantine {} could not be made: {error}", dir.display()))?;
    let guard = RemovedOnDrop { dir, remove };
    Ok(act(&quarantined_git(git, &guard.dir, objects)))
}

/// The quarantine directory, which calls its remover on it when dropped, including on unwind.
/// with_object_quarantine's remover is remove_dir_all, whose failure leaves the directory in the OS temp dir,
/// outside the repository.
struct RemovedOnDrop<R: FnMut(&Path)> {
    dir: PathBuf,
    remove: R,
}

impl<R: FnMut(&Path)> Drop for RemovedOnDrop<R> {
    fn drop(&mut self) {
        (self.remove)(&self.dir);
    }
}

#[cfg(test)]
#[path = "with_object_quarantine_tests.rs"]
mod tests;
