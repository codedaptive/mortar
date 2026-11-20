//! FORMATTER_STDERR_CAP: how much of a run_in_group command's stderr its end keeps (read_capped and
//! decide_group_run_end cut at it). A constant only; decide_group_run_end_tests and read_capped_tests exercise it
//! through a stderr longer than the cap.

/// In bytes. Bounds what an outcome carries however much a command writes; the start of stderr is kept because it
/// names the first failure.
pub(crate) const FORMATTER_STDERR_CAP: usize = 4096;
