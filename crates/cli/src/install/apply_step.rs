//! apply_step: Applies one install step: a sibling temp `.<name>.<token>.mortar-install` is written, then renamed
//! over the step's path, so the path holds the old thing or the new one, never half of either. The temp is created
//! exclusively under a name from `token` (fresh_temp), so a link planted at a temp name is never followed or
//! replaced. The step's folder already exists (apply_install makes it first).
//! No sibling test: it touches the disk; the install integration tests in install/integration/ cover it (the placed binary's
//! bytes, mode and single link, the `mr` symlink, the skill files). temp_files.rs plants a link at the first token's
//! temp name and checks that this file's own creation of each step's temp (binary, link, skill) leaves the planted
//! link and its target as they were.

use std::fs::{self, File, OpenOptions, Permissions};
use std::io::{self, Write};
use std::os::unix::fs::{symlink, OpenOptionsExt, PermissionsExt};
use std::path::Path;

use super::{fresh_temp, InstallStep};

/// Ok once the step's path holds what the step says: PlaceBinary copies `from`'s bytes and sets mode 0o755; Link
/// makes a symlink (never a hard link) to `target`; WriteSkill writes `text`. `token` names the temps tried (install
/// passes random_token). Err "cannot <place|link|write> <path>: <error>", with the temp removed if one was made.
pub fn apply_step(step: &InstallStep, token: &mut dyn FnMut() -> u64) -> Result<(), String> {
    let (verb, path) = match step {
        InstallStep::PlaceBinary { to, .. } => ("place", to),
        InstallStep::Link { at, .. } => ("link", at),
        InstallStep::WriteSkill { path, .. } => ("write", path),
    };
    replace(step, path, token).map_err(|error| format!("cannot {verb} {}: {error}", path.display()))
}

/// Makes the temp, fills it, renames it over `path`; on an error after the temp exists, removes that temp, which is
/// ours because it was created exclusively.
fn replace(step: &InstallStep, path: &Path, token: &mut dyn FnMut() -> u64) -> io::Result<()> {
    let (temp, file) = fresh_temp(path, token, &mut |temp| match step {
        InstallStep::PlaceBinary { .. } => new_file(temp, 0o755).map(Some),
        InstallStep::Link { target, .. } => symlink(target, temp).map(|()| None),
        InstallStep::WriteSkill { .. } => new_file(temp, 0o666).map(Some),
    })?;
    let written = file.map_or(Ok(()), |file| fill(step, file)).and_then(|()| fs::rename(&temp, path));
    if written.is_err() {
        // The step's own error is the one to report; a temp that is already gone is no second error.
        let _ = fs::remove_file(&temp);
    }
    written
}

/// A new file at `temp`, failing with AlreadyExists if anything (a planted link included) is already there.
fn new_file(temp: &Path, mode: u32) -> io::Result<File> {
    OpenOptions::new().write(true).create_new(true).mode(mode).open(temp)
}

/// Writes the step's content into its open temp. The binary's mode is set on the open handle, never by path, and
/// again after the copy because the creation mode is narrowed by the umask.
fn fill(step: &InstallStep, mut file: File) -> io::Result<()> {
    match step {
        InstallStep::PlaceBinary { from, .. } => {
            io::copy(&mut File::open(from)?, &mut file)?;
            file.set_permissions(Permissions::from_mode(0o755))
        }
        InstallStep::WriteSkill { text, .. } => file.write_all(text.as_bytes()),
        InstallStep::Link { .. } => Ok(()),
    }
}
