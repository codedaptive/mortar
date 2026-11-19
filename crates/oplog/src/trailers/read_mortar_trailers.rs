//! Reading a message's Mortar trailers through git; pick_mortar_trailers makes every decision.
//! No unit test: it runs git. Integration tests should cover: "read_mortar_trailers reads what
//! write_mortar_trailers wrote", "read_mortar_trailers on a message with no trailers is all absent" and "a
//! body with a `---` line still has Mortar's trailers read".

use gitio::Git;

use super::{pick_mortar_trailers, MortarTrailerError, MortarTrailers};

/// The Mortar trailers of `message`; Git when git fails, ChangeId when a Mortar-Change-Id is malformed.
pub fn read_mortar_trailers(git: &Git, message: &str) -> Result<MortarTrailers, MortarTrailerError> {
    let pairs = git.trailers(message).map_err(MortarTrailerError::Git)?;
    pick_mortar_trailers(&pairs).map_err(MortarTrailerError::ChangeId)
}
