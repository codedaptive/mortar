//! Makes the drive's throwaway folder. Real file I/O: the shell step (steps/shell.rs) covers it through World::new
//! on every drive.

use std::fs;
use std::io;
use std::sync::atomic::{AtomicU32, Ordering};

use super::Scratch;

static NEXT: AtomicU32 = AtomicU32::new(0);

impl Scratch {
    /// Takes the next free `<n>`. The path is canonical (macOS's temp folder sits behind a symlink), so it reads
    /// the same as the paths git prints. If it cannot be made canonical, the folder just made is removed again.
    pub fn new() -> io::Result<Scratch> {
        loop {
            let n = NEXT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!("mortar-accept-{}-{n}", std::process::id()));
            match fs::create_dir(&path) {
                Ok(()) => {
                    return fs::canonicalize(&path).map(|path| Scratch { path }).inspect_err(|_| {
                        // The canonicalize error is the one reported; the folder is still empty.
                        let _ = fs::remove_dir(&path);
                    });
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
    }
}
