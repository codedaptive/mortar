//! Output: what a finished git run produced.

/// The exit code and the raw bytes git wrote.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    /// The exit code: 0, or one of the run's answer codes.
    pub status: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}
