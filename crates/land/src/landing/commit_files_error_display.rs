//! Display for CommitFilesError: its own refusals say why and that nothing was written; Paths and Land show the inner
//! error's text unchanged. commit_files_error_display_tests covers every variant.

use crate::landing::CommitFilesError;
use std::fmt;

impl fmt::Display for CommitFilesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommitFilesError::NotLanding => {
                write!(f, "there is no landing in flight: nothing landed since the last publish, nothing written")
            }
            CommitFilesError::NotLanded { landed, integration } => write!(
                f,
                "integration is at {integration}, not at {landed}, the commit the landing state names; nothing was \
                 written"
            ),
            CommitFilesError::FolderOffTip { folder, integration } => {
                let folder = folder.as_ref().map_or("no commit".to_string(), |commit| commit.to_string());
                write!(
                    f,
                    "the integration folder is at {folder}, not at integration's tip {integration}; nothing written"
                )
            }
            CommitFilesError::Paths(e) => write!(f, "{e}"),
            CommitFilesError::Land(e) => write!(f, "{e}"),
        }
    }
}
