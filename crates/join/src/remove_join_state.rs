//! remove_join_state: removes mortar-join.json once a resolve has finished.
//! No unit test: it removes a real file. Integration tests are to cover, in a throwaway repository: a clean resolve
//! removes mortar-join.json; a next run repeated after a clean finish finds no state.

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
