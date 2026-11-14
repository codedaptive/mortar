//! GuardVerdict: what the guard decided. A bare type (S-7); guard_decide_tests exercises it.

/// The caller (`mortar guard-hook`) prints a refusal's three parts and exits non-zero, so git aborts the
/// transaction or the push.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuardVerdict {
    Allow,
    /// `why` names the ref refused (or the input line the guard could not read), `owner` who may move it, `next` what
    /// to do instead.
    Refuse { why: String, owner: String, next: String },
}
