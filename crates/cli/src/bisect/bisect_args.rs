//! What `mr bisect` reads from its line.
//! A bare type (S-7); parse_bisect_args_tests exercises it.

use std::time::Duration;

use gitio::Oid;
use land::TestCommand;

/// The two ends of the range, each probe's time limit, and the test command given after `--`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BisectArgs {
    pub last_pass: Oid,
    pub failed: Oid,
    pub timeout: Duration,
    pub command: TestCommand,
}
