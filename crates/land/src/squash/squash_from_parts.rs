//! squash_from_parts: a Squash from what read_squash reads off its commit, or the refusal that names what is wrong.
//! squash_from_parts_tests covers each decision.

use crate::squash::{last_trailer_value, squash_message_requests, Squash, SquashError, MORTAR_SQUASH_OF_KEY};
use gitio::Oid;
use oplog::{ChangeIdError, MortarTrailers, MORTAR_CHANGE_ID_KEY};

/// `commit`'s Squash from its picked Mortar trailers, its trailer pairs and its message. A malformed
/// Mortar-Change-Id is TrailerMalformed naming its text, any other pick error ChangeId; a missing Mortar-Change-Id
/// or Mortar-Squash-Of is TrailerMissing; a Mortar-Squash-Of that is not a commit id is TrailerMalformed; request
/// lines that do not match the subject's count are RequestsMalformed. Mortar-Zone is not read: Squash has no zone.
pub(crate) fn squash_from_parts(
    commit: Oid,
    picked: Result<MortarTrailers, ChangeIdError>,
    trailers: &[(String, String)],
    message: &str,
) -> Result<Squash, SquashError> {
    let missing = |trailer: &str| SquashError::TrailerMissing { commit: commit.clone(), trailer: trailer.to_string() };
    let malformed = |trailer: &str, value: String| SquashError::TrailerMalformed {
        commit: commit.clone(),
        trailer: trailer.to_string(),
        value,
    };
    let change_id = match picked {
        Ok(trailers) => trailers.change_id.ok_or_else(|| missing(MORTAR_CHANGE_ID_KEY))?,
        Err(ChangeIdError::Malformed { text, .. }) => return Err(malformed(MORTAR_CHANGE_ID_KEY, text)),
        Err(e) => return Err(SquashError::ChangeId(e)),
    };
    let value = last_trailer_value(trailers, MORTAR_SQUASH_OF_KEY).ok_or_else(|| missing(MORTAR_SQUASH_OF_KEY))?;
    let source = Oid::parse(value).map_err(|_| malformed(MORTAR_SQUASH_OF_KEY, value.to_string()))?;
    let requests =
        squash_message_requests(message).ok_or_else(|| SquashError::RequestsMalformed { commit: commit.clone() })?;
    Ok(Squash { commit, change_id, source, requests })
}
