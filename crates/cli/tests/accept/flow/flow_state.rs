//! What step 1's check groups hand on to the later ones. Data only: run_flow.rs fills it group by group.

use std::path::PathBuf;

use crate::stage::Stage;

/// The set-up; the request folders r1, r2, r3 (zone b) and r4; zone a's tip after r1's edit of src/a/a.txt was
/// joined (check 5), whose tree check 8's conflicting commit reuses; whether r3's change was joined into zone b
/// before the squash, and what that join said (check 14); the squash's change ID and its first and second commits
/// (checks 7 and 9).
#[derive(Debug, Default)]
pub struct FlowState {
    pub stage: Stage,
    pub r1: PathBuf,
    pub r2: PathBuf,
    pub r3: PathBuf,
    pub r4: PathBuf,
    pub r1_edited: String,
    pub r3_joined: bool,
    pub r3_saw: String,
    pub change_id: String,
    pub first: String,
    pub second: String,
}
