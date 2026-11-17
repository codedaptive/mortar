//! The titles of 01-flow.md's fifteen checks, in order. A const only: not_run_tests.rs reads it through not_run.

/// Check n's title is at index n - 1.
pub const FLOW_CHECKS: [&str; 15] = [
    "requests made",
    "saved",
    "joined",
    "a zone moves under a request",
    "a conflict",
    "handed over and resolved",
    "squash made",
    "checked for merging",
    "re-reviewed after an edit",
    "landed",
    "a failed integration test",
    "publish refused when origin moved",
    "published",
    "the other zone replayed",
    "plain git throughout",
];
