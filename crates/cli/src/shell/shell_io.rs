//! The two reads run_in makes outside itself, given as functions so its unit tests can fake them.

use std::path::Path;

use gitio::{GitError, Version};

use crate::context::{ContextError, FolderContext};

/// `read_version` answers the git floor check; in production it is `|| Git::new(cwd).version()`, which runs
/// `git version` and applies gitio's floor. `read_folder` reads an agent verb's folder; in production it is
/// context::read_folder_context.
pub(crate) struct ShellIo<'a> {
    pub read_version: &'a dyn Fn() -> Result<Version, GitError>,
    pub read_folder: &'a dyn Fn(&Path) -> Result<FolderContext, ContextError>,
}
