//! Where a lock lives: `<common>/mortar/locks/<lock file name>`. A path join only; touches no file.

use std::path::{Path, PathBuf};

use super::{lock_file_name, LockError, LockName};

/// The lock file's path under the git common directory `common`.
pub fn lock_path(common: &Path, name: &LockName) -> Result<PathBuf, LockError> {
    Ok(common.join("mortar").join("locks").join(lock_file_name(name)?))
}
