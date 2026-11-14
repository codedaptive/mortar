//! SquashRequest: one request's line of a squash message. A declaration only (S-7); squash_requests_tests
//! and squash_message_tests exercise it.

/// A Mortar-Request value and the subjects of its commits, oldest first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SquashRequest {
    pub(crate) request: String,
    pub(crate) subjects: Vec<String>,
}
