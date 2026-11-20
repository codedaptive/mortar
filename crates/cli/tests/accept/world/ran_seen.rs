//! How a child ended, as a check line shows it.

use super::Ran;

impl Ran {
    /// "exit <code>", "killed by signal <n>" or "did not run"; unless the exit is 0, then " / " and the first
    /// stderr line that is not blank.
    pub fn seen(&self) -> String {
        let ended = match (self.code, self.signal) {
            (Some(code), _) => format!("exit {code}"),
            (None, Some(signal)) => format!("killed by signal {signal}"),
            (None, None) => "did not run".to_string(),
        };
        let first = self.stderr.lines().map(str::trim).find(|line| !line.is_empty());
        match first {
            Some(line) if self.code != Some(0) => format!("{ended} / {line}"),
            _ => ended,
        }
    }
}
