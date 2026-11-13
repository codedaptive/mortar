//! Every way reading Mortar's trailers can fail.
//! A bare type (S-7); mortar_trailer_error_display_tests and mortar_trailer_error_source_tests exercise each
//! variant.

use gitio::GitError;

use crate::change_id::ChangeIdError;

/// A failure reading a message's Mortar trailers.
#[derive(Debug)]
pub enum MortarTrailerError {
    /// git could not read the message's trailers.
    Git(GitError),
    /// A Mortar-Change-Id trailer is not a change ID.
    ChangeId(ChangeIdError),
}
