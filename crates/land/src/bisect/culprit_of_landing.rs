//! Naming the culprit from a landing commit: its squash's change ID and zone (S-57, S-58).

use gitio::Oid;
use oplog::{MortarTrailerError, MortarTrailers};

use super::{Culprit, CulpritError};

/// The Culprit for landing `commit`, whose parents in git's order are `parents`.
///
/// One parent is a fast-forward landing: the squash is `commit` itself. Two are a merge landing with parents
/// [integration, squash] (S-58): the squash is the second parent. Any other count is NotALanding, refused
/// before `trailers_of` is called. `trailers_of` is called once, on the squash; its error is Trailers. The
/// Culprit names `commit`, never the squash, so it stays on origin's first-parent line.
pub fn culprit_of_landing(
    commit: &Oid,
    parents: &[Oid],
    trailers_of: impl FnOnce(&Oid) -> Result<MortarTrailers, MortarTrailerError>,
) -> Result<Culprit, CulpritError> {
    let squash = match parents {
        [_] => commit,
        [_, squash] => squash,
        _ => return Err(CulpritError::NotALanding { commit: commit.clone(), parents: parents.len() }),
    };
    let trailers = trailers_of(squash).map_err(CulpritError::Trailers)?;
    Ok(Culprit { commit: commit.clone(), change_id: trailers.change_id, zone: trailers.zone })
}
