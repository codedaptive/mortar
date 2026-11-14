//! A ref that moves only forward, and who protects it.
//! A bare type (S-7); protected_move_verdict_tests exercises it.

/// `refname` is the ref's full name; `by` names who protects it, shown in the refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtectedRef {
    pub refname: String,
    pub by: String,
}
