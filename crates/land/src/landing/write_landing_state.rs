//! write_landing_state: writes `<common>/mortar/landing.json`; a killed process leaves the old state or the new one.
//! No unit test (S-5): it creates a folder and writes and renames real files.
//! Covered by the landing_zit tests a_squash_on_origin_fast_forwards_integration_to_the_squash (landing.json
//! holds exactly the state), a_squash_behind_a_moved_origin_lands_as_a_merge_commit and
//! killed_between_state_and_op_recovers_and_the_repeat_lands_the_states_commit (landing.json written, integration
//! unchanged).

use crate::landing::{encode_landing_state, landing_state_path, LandingState, LandingStateError};
use std::path::Path;

/// Creates `<common>/mortar` if it is absent, writes the encoded state to `landing.json.tmp` there, then renames it
/// over `landing.json`, so a reader finds the old state or the new one, never a torn one. A failure is Write naming
/// the folder, the temp file or the state file, whichever step failed.
pub(crate) fn write_landing_state(common: &Path, state: &LandingState) -> Result<(), LandingStateError> {
    let path = landing_state_path(common);
    let folder = common.join("mortar");
    std::fs::create_dir_all(&folder).map_err(|source| LandingStateError::Write { path: folder.clone(), source })?;
    let temp = folder.join("landing.json.tmp");
    std::fs::write(&temp, encode_landing_state(state))
        .map_err(|source| LandingStateError::Write { path: temp.clone(), source })?;
    std::fs::rename(&temp, &path).map_err(|source| LandingStateError::Write { path, source })
}
