//! SaveHooksError: why save_with_hooks or save_from_snapshot_with_hooks did not save. It replaces SnapshotError as
//! save's error once cli has moved to the hooked twins, and a later snapshot order removes the hookless entry
//! points. A declaration and its From only, with no std::error::Error impl as SnapshotError has none: its Display
//! is save_hooks_error_display, tested per variant in save_hooks_error_display_tests.rs. The integration tests in
//! tests/snapshot/save_cases.rs and save_from_snapshot_cases.rs get HookRefused (listed in save_with_hooks.rs and
//! save_from_snapshot_with_hooks.rs).

use crate::SnapshotError;

/// Derives Debug only, as SnapshotError does.
#[derive(Debug)]
pub enum SaveHooksError {
    /// Any error the hookless twin (save_with_snapshot or save_from_snapshot) gives, unchanged.
    Snapshot(SnapshotError),
    /// A pre-commit or commit-msg hook refused the save's commit: no save commit was written, no branch moved, no
    /// save op was logged and the folder's index is as it was; a snapshot step that wrote keeps its snapshot ref and
    /// its op. The refusal carries the hook's name, exit code, stdout and stderr.
    HookRefused(gitio::HookRefusal),
}

impl From<SnapshotError> for SaveHooksError {
    fn from(error: SnapshotError) -> Self {
        SaveHooksError::Snapshot(error)
    }
}
