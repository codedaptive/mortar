//! Partner: a partner zone and its tip. A declaration only; encode_join_state_tests and decode_join_state_tests
//! exercise it inside a RequestBase.

use gitio::Oid;

/// A partner zone, by name (its ref is `refs/heads/zone/<zone>`), and its tip.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Partner {
    pub zone: String,
    pub tip: Oid,
}
