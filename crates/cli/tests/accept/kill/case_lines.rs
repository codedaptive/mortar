//! One case's check titles as it writes them. Data only: its methods are in case_lines_new.rs,
//! case_lines_check.rs and case_lines_not_run.rs; case_lines_tests.rs covers them.

/// The case's own titles, then the consistency items, each led by the case's label; `own` is how many are the
/// case's own, and `next` the title the next line takes.
#[derive(Debug, Clone, PartialEq)]
pub struct CaseLines {
    pub titles: Vec<String>,
    pub own: usize,
    pub next: usize,
}
