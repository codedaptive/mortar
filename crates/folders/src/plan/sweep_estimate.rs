//! What a sweep would free, as the caller supplies it; a below-floor refusal reports it.
//! A bare type; below_floor_display_tests exercises it.

/// `bytes` is the figure; `reason`, when given, says why the figure is what it is (for example,
/// that it was not measured).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SweepEstimate {
    pub bytes: u64,
    pub reason: Option<String>,
}
