//! FORMATTER_STDERR_CAP: how much of a failed or timed-out formatter's stderr an outcome keeps. A constant only;
//! decide_formatter_result_tests exercises it through a stderr longer than the cap.

/// In bytes. Bounds what an outcome carries however much a formatter writes; the start of stderr is kept because it
/// names the first failure.
pub(crate) const FORMATTER_STDERR_CAP: usize = 4096;
