//! write_record: Writes install's record: a sibling temp file, then a rename. The temp is created exclusively under a
//! name from `token` (fresh_temp), so a link planted at a temp name is never followed or replaced.
//! No sibling test: it touches the disk. The install integration tests in install/integration/ cover it: uninstall.rs
//! (uninstall_restores_home_after_install reads back the record install wrote), placement.rs, and temp_files.rs,
//! which plants a link at the first token's temp name and checks this file's own creation leaves it and its target
//! as they were, and that eight taken names fail with AlreadyExists and change nothing.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use super::fresh_temp;

/// Writes `text` at `path` through a sibling temp file, `.<name>.<token>.mortar-install`, renamed over it, so a
/// reader sees the old record or the new one, never part of one. `token` names the temps tried (install passes
/// random_token). On an error after the temp exists, that temp is removed.
pub fn write_record(path: &Path, text: &str, token: &mut dyn FnMut() -> u64) -> io::Result<()> {
    let (temp, mut file) =
        fresh_temp(path, token, &mut |temp| OpenOptions::new().write(true).create_new(true).open(temp))?;
    let written = file.write_all(text.as_bytes()).and_then(|()| fs::rename(&temp, path));
    if written.is_err() {
        // The write's own error is the one to report; a temp file that is already gone is no second error.
        let _ = fs::remove_file(&temp);
    }
    written
}
