//! The Done steps: each step's spec and body, the decision whether it runs, and the drive over them all.

mod agent;
mod decide_step;
mod decision;
mod disk;
mod empty_log;
mod done_list;
mod flow;
mod gate;
mod guard;
mod kill;
mod run_drive;
mod run_step;
mod shell;
mod step_body;
mod step_spec;
mod step_verdict;

#[cfg(test)]
mod decide_step_tests;
#[cfg(test)]
mod empty_log_tests;
#[cfg(test)]
mod run_step_tests;
#[cfg(test)]
mod step_verdict_tests;

pub use agent::AGENT;
pub use decide_step::decide_step;
pub use decision::Decision;
pub use disk::DISK;
pub use empty_log::empty_log;
pub use done_list::DONE_LIST;
pub use flow::FLOW;
pub use gate::Gate;
pub use guard::GUARD;
pub use kill::KILL;
pub use run_drive::run_drive;
pub use run_step::run_step;
pub use shell::SHELL;
pub use step_body::StepBody;
pub use step_spec::StepSpec;
pub use step_verdict::step_verdict;
