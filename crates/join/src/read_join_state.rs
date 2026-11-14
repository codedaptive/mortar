//! read_join_state: reads mortar-join.json back into a JoinState, or None when there is none.
//! No unit test: it reads a real file (S-5); decode_join_state_tests covers the text it accepts and refuses.
//! Nagatha's integration tests are to cover, in a throwaway repository: the state a conflicted join wrote to the
//! folder's git dir is read back by resolve (WO-join-3 Goal 1; S-21); with no state file, resolve refuses
//! (WO-join-3 Goal 1: "No state: refuse, saying there is nothing to resolve").

use crate::decode_join_state::decode_join_state;
use crate::join_state::JoinState;
use crate::JoinRunError;
use std::io::ErrorKind;
use std::path::Path;

/// Reads and decodes the state file at `path`. No file is None. A file that cannot be read (one that is not UTF-8
/// included) is StateRead; text not in the layout encode_join_state writes is StateCorrupt with the reason.
pub(crate) fn read_join_state(path: &Path) -> Result<Option<JoinState>, JoinRunError> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(source) if source.kind() == ErrorKind::NotFound => return Ok(None),
        Err(source) => return Err(JoinRunError::StateRead { path: path.to_path_buf(), source }),
    };
    decode_join_state(&text).map(Some).map_err(|reason| JoinRunError::StateCorrupt { path: path.to_path_buf(), reason })
}
