//! RequestBase: a request's base and the partners it was made with. A declaration only; encode_join_state_tests,
//! decode_join_state_tests and after_replay_tests exercise it.

use crate::partner::Partner;
use gitio::Oid;

/// A request's base and the partners it was made with (the folder record's base and partners), in the record's
/// order. With no partners, `base` is a plain zone commit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestBase {
    pub base: Oid,
    pub partners: Vec<Partner>,
}
