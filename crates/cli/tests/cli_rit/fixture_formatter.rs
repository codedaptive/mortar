//! Formatter arguments supplied to the real init binary by the RIT fixtures; no cli config API.
#[derive(Clone)]
pub struct FixtureFormatter {
    pub command: Vec<String>,
    pub timeout_seconds: u64,
}
