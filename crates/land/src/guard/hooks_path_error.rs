//! Why check_hooks_path refused or could not decide.
//! A bare type; hooks_path_error_display_tests checks the Relative text, hooks_path_error_source_tests each
//! variant's source. tests/land_rit/check_hooks_path.rs checks the Git text and source through gitio's Display.

use std::path::PathBuf;

use gitio::GitError;

/// Why check_hooks_path refused or could not decide.
#[derive(Debug)]
pub enum HooksPathError {
    /// git could not read core.hooksPath.
    Git(GitError),
    /// core.hooksPath as git reads it (`--type=path`) is not absolute; `value` is that value.
    Relative { value: PathBuf },
}
