//! Which `mr` a shell would run (S-133 (1)): the first on PATH, whether it is this build (its canonical path, or the
//! Mortar marker in its bytes, S-147), and the warning `mr guard install` prints when it is not.

mod file_has_marker;
mod first_on_path;
mod marker_in_chunks;
mod mortar_marker;
mod mr_path_warning;
mod path_entries;
mod path_warning_for;
mod same_build;

#[cfg(test)]
mod marker_in_chunks_tests;
#[cfg(test)]
mod mr_path_warning_tests;
#[cfg(test)]
mod path_entries_tests;
#[cfg(test)]
mod path_warning_for_tests;
#[cfg(test)]
mod same_build_tests;

pub use file_has_marker::file_has_marker;
pub use first_on_path::first_on_path;
pub use marker_in_chunks::marker_in_chunks;
pub use mortar_marker::MORTAR_MARKER;
pub use mr_path_warning::mr_path_warning;
pub use path_entries::path_entries;
pub use path_warning_for::path_warning_for;
pub use same_build::same_build;
