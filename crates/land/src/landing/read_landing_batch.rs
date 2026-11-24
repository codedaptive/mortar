//! read_landing_batch: the landing batch's members in landing order, for cli (which member a landing is, and the
//! zones a publish is for). No unit test: it reads a file (S-5); batch_members_tests covers the members it names.
//! Covered by landing_zit's batch::read_landing_batch_names_the_members_in_landing_order.

use crate::landing::{batch_members, read_landing_state, BatchMember, LandingStateError};
use gitio::Git;

/// No landing state is None; otherwise every member's change ID and zone, earlier members first. Resolving a pending
/// commit_files commit (resolve_landing_state) changes only the newest member's integration, never a member's change
/// ID or zone, so the answer is the resolved state's members without reading integration's ref. Read only: it
/// writes nothing and reads nothing else. LandingStateError has no variant for git, so finding the common dir
/// failing is Read, with the repository's path and the git error as its source; reading the state failing is
/// Read or Corrupt as read_landing_state says.
pub fn read_landing_batch(git: &Git) -> Result<Option<Vec<BatchMember>>, LandingStateError> {
    let common = git
        .common_dir()
        .map_err(|error| LandingStateError::Read { path: git.repo.clone(), source: std::io::Error::other(error) })?;
    Ok(read_landing_state(&common)?.map(|state| batch_members(&state)))
}
