//! Who asks for an undo or a restore, and why.
//! A bare type (S-7); caller_op_tests exercises it.

/// `key` makes a repeat safe, as Op's key does; `actor` and `reason` go into the new op unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpCaller {
    pub key: String,
    pub actor: String,
    pub reason: String,
}
