//! decide_commit_files: whether commit_files may commit onto integration, from what it read. decide_commit_files_tests
//! covers every branch.

use crate::landing::{resolve_landing_state, CommitFilesError, LandingState};
use gitio::Oid;

/// `integration` is integration's tip and `folder` the integration folder's HEAD commit (None when unborn). No state:
/// NotLanding. The state resolved against `integration` (resolve_landing_state) whose newest member names another
/// commit: NotLanded. `folder` not at `integration`: FolderOffTip. Otherwise the resolved state, whose newest
/// member's integration is `integration`, so the caller writes its next pending commit on top of the tip.
pub(crate) fn decide_commit_files(
    state: Option<LandingState>,
    integration: &Oid,
    folder: Option<&Oid>,
) -> Result<LandingState, CommitFilesError> {
    let state = resolve_landing_state(state.ok_or(CommitFilesError::NotLanding)?, integration);
    if &state.newest.integration != integration {
        return Err(CommitFilesError::NotLanded { landed: state.newest.integration, integration: integration.clone() });
    }
    if folder != Some(integration) {
        return Err(CommitFilesError::FolderOffTip { folder: folder.cloned(), integration: integration.clone() });
    }
    Ok(state)
}
