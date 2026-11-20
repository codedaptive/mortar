//! read_landing_state: reads `<common>/mortar/landing.json` back into a LandingState, or None when there is none.
//! No unit test: it reads a real file; decode_landing_state's text cases are repo integration tests (RITs):
//! tests/land_rit/state_codec.rs covers them. Covered by the landing_zit tests
//! a_second_landing_is_busy_and_the_first_ones_repeat_is_landed_with_the_log_unchanged,
//! back_out_restores_integration_then_has_nothing_to_back_out and
//! killed_between_state_and_op_recovers_and_the_repeat_lands_the_states_commit, and the publish_zit tests
//! publish_moves_the_remote_then_the_local_origin_and_cleans_up,
//! killed_between_the_steps_recovers_and_the_repeat_publishes_once and
//! with_no_landing_state_there_is_nothing_to_publish_and_the_remote_is_untouched, which read the state land wrote or
//! find none.

use crate::landing::{decode_landing_state, landing_state_path, LandingState, LandingStateError};
use std::io::ErrorKind;
use std::path::Path;

/// Reads and decodes the landing state under `common`. No file is None. A file that cannot be read (one that is
/// not UTF-8 included) is Read; text not in the layout encode_landing_state writes is Corrupt with the reason.
pub(crate) fn read_landing_state(common: &Path) -> Result<Option<LandingState>, LandingStateError> {
    let path = landing_state_path(common);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(source) if source.kind() == ErrorKind::NotFound => return Ok(None),
        Err(source) => return Err(LandingStateError::Read { path, source }),
    };
    decode_landing_state(&text).map(Some).map_err(|reason| LandingStateError::Corrupt { path, reason })
}
