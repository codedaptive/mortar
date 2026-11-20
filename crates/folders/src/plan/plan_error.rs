//! A refusal from a folder's plan: a name, a path or a list that cannot be used.
//! A bare type; plan_error_display_tests exercises it.

/// `reason` names what was refused and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanError {
    pub reason: String,
}
