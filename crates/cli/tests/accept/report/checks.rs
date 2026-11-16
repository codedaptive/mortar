//! The lines a step's body records while it runs. Data only: its methods are in checks_new.rs, checks_check.rs,
//! checks_log.rs and checks_lines.rs; checks_new_tests.rs and checks_check_tests.rs cover them.

use super::Line;

/// Collects a step's check and log lines in order; a step body receives it and run_step turns it into a block.
#[derive(Debug, Default)]
pub struct Checks {
    pub(super) lines: Vec<Line>,
}
