//! CommitFilesError: why commit_files was refused. A bare type; commit_files_error_display_tests exercises its texts,
//! commit_files_error_source_tests its source, decide_commit_files_tests the refusals decided before anything runs.

use crate::landing::LandError;
use gitio::Oid;

/// A commit_files refusal. Debug only: CommitPathsError and LandError are neither Clone nor Eq.
#[derive(Debug)]
pub enum CommitFilesError {
    /// No landing state: nothing has landed since the last publish, so there is no landing to add the paths to.
    /// Nothing was written.
    NotLanding,
    /// A landing state exists but integration is not at the commit it names (for example a land killed before its op
    /// left integration at origin). Nothing was written.
    NotLanded {
        /// The commit the landing state names as the landing's integration.
        landed: Oid,
        /// Integration's tip.
        integration: Oid,
    },
    /// The integration folder's HEAD is not at integration's tip, so its files are not the landing's. Nothing was
    /// written.
    FolderOffTip {
        /// The commit the folder's HEAD points at; None when it is unborn.
        folder: Option<Oid>,
        /// Integration's tip.
        integration: Oid,
    },
    /// snapshot's commit_paths refused the paths or failed, in its own words.
    Paths(snapshot::CommitPathsError),
    /// A git run (reading integration's tip, the common folder or the folder's HEAD) failed, the landing state could
    /// not be read or written, or the op that moves integration failed or was refused.
    Land(LandError),
}
