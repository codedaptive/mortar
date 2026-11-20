//! One write install makes, decided before any is made.
//! A bare type; the planning tests and step_record_tests exercise it.

use std::path::PathBuf;

/// PlaceBinary: copy the binary at `from` to `to`. Link: a symlink at `at` pointing at `target`. WriteSkill: write
/// `text` as the skill file at `path`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallStep {
    PlaceBinary { from: PathBuf, to: PathBuf },
    Link { at: PathBuf, target: PathBuf },
    WriteSkill { path: PathBuf, text: String },
}
