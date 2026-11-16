//! The drive's own throwaway folder. Data only: its methods are in scratch_new.rs, scratch_path.rs,
//! scratch_close.rs and scratch_drop.rs, real file I/O the shell step (steps/shell.rs) covers through World::new
//! and World::close on every drive.

use std::path::PathBuf;

/// A fresh folder `mortar-accept-<pid>-<n>` under the system temp folder. It removes only the folder it made:
/// `close` removes it and reports an error; dropping it without `close` removes it best-effort. `path` is empty
/// once `close` has taken it.
pub struct Scratch {
    pub(super) path: PathBuf,
}
