//! remove_join_state: removes mortar-join.json once a resolve has finished.
//! No unit test: it removes a real file (S-5). Nagatha's integration tests are to cover, in a throwaway repository:
//! a clean resolve removes mortar-join.json (WO-join-3 Goal 3: "mortar-join.json is removed"); a repeat after that
//! finds no state (WO-join-3 Goal 4: "a next run repeated after a clean finish finds no state").

use crate::JoinRunError;
use std::io::ErrorKind;
use std::path::Path;

/// Removes the state file at `path`. A file already gone is done, so a repeat after a kill succeeds; any other
/// failure is StateRemove.
pub(crate) fn remove_join_state(path: &Path) -> Result<(), JoinRunError> {
    match std::fs::remove_file(path) {
        Err(source) if source.kind() != ErrorKind::NotFound => {
            Err(JoinRunError::StateRemove { path: path.to_path_buf(), source })
        }
        _ => Ok(()),
    }
}
