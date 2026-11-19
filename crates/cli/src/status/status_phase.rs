//! StatusPhase: where a request folder stands; status_outcome names it and status_next gives its step.
//! A bare type; status_phase_from_status_state_tests exercises it.

/// The one phase a request folder is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusPhase {
    /// A folder move a kill or a refused move left undone.
    Stalled,
    /// A join stopped on a conflict.
    Paused,
    /// Work in the folder not yet saved.
    Unsaved,
    /// Everything saved, not yet joined to the zone.
    Saved,
    /// Saved and joined to the zone.
    Joined,
}
