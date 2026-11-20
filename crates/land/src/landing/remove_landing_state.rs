//! remove_landing_state: removes `<common>/mortar/landing.json` once a landing is backed out or published.
//! No unit test: it removes a real file. Covered by the landing_zit test
//! back_out_restores_integration_then_has_nothing_to_back_out (landing.json gone) and the publish_zit tests
//! publish_moves_the_remote_then_the_local_origin_and_cleans_up (landing.json gone) and
//! a_remote_moved_by_someone_else_is_fetched_backed_out_onto_and_lands_again (landing.json gone).

use crate::landing::{landing_state_path, LandingStateError};
use std::io::ErrorKind;
use std::path::Path;

/// Removes the landing state under `common`. A file already gone is done, so a repeat after a kill succeeds; any
/// other failure is Remove.
pub(crate) fn remove_landing_state(common: &Path) -> Result<(), LandingStateError> {
    let path = landing_state_path(common);
    match std::fs::remove_file(&path) {
        Err(source) if source.kind() != ErrorKind::NotFound => Err(LandingStateError::Remove { path, source }),
        _ => Ok(()),
    }
}
