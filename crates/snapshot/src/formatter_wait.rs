//! FormatterWait: what to do next with a running formatter. A declaration only; decide_formatter_wait_tests
//! exercises it.

/// The step decide_formatter_wait gives run_formatter_child on each poll.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FormatterWait {
    /// Still running inside the limit: poll again.
    Wait,
    /// Not over at or past the limit: stop waiting; the formatter is killed if it is still running.
    Kill,
    /// The run is over (formatter reaped and its stderr at EOF): keep its result.
    Done,
}
