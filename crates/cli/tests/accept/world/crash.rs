//! Where the kill shim kills. Data only: its method is in crash_as_str.rs; the shell step (steps/shell.rs) kills
//! at both points on every drive.

/// Before: the armed update-ref never runs. After: it runs, then the kill.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Crash {
    Before,
    After,
}
