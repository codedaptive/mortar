//! Writing Mortar's trailers onto a message through git, last (S-10); mortar_trailer_pairs makes the pairs.
//! No unit test: it runs git (S-5). Covered by Nagatha's integration group: "write_mortar_trailers then
//! read_mortar_trailers round trip" and "a message already holding an older Mortar-Change-Id reads back the
//! one written last".

use gitio::{Git, GitError};

use super::{mortar_trailer_pairs, MortarTrailers};

/// `message` with the present Mortar trailers added after its existing trailers.
pub fn write_mortar_trailers(git: &Git, message: &str, trailers: &MortarTrailers) -> Result<String, GitError> {
    git.with_trailers(message, &mortar_trailer_pairs(trailers))
}
