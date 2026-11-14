//! snapshot: see the Mortar mission (Layout).

mod candidate_size;
mod cap_candidate;
mod cap_candidates;
mod cap_mismatch;
mod cap_refusals;
mod capped_excludes;
mod child_exit;
mod decide_formatter_result;
mod decide_formatter_wait;
mod decide_snapshot;
mod default_size_cap;
mod empty_tree;
mod file_size;
mod folder_tree;
mod formatter;
mod formatter_argv;
mod formatter_paths;
mod formatter_result;
mod formatter_run;
mod formatter_stderr_cap;
mod formatter_wait;
mod is_saved;
mod literal_exclude;
mod previous_snapshot;
mod read_previous_snapshot;
mod refused_file;
mod run_formatter;
mod run_formatter_child;
mod snapshot;
mod snapshot_decision;
mod snapshot_error;
mod snapshot_excludes;
mod snapshot_formatted;
mod snapshot_from_apply;
mod snapshot_op;
mod snapshot_outcome;
mod snapshot_parents;
mod snapshot_ref_name;
mod snapshot_target;
mod snapshot_trailers;

pub use default_size_cap::DEFAULT_SIZE_CAP;
pub use formatter::Formatter;
pub use formatter_result::FormatterResult;
pub use formatter_run::FormatterRun;
pub use is_saved::is_saved;
pub use refused_file::RefusedFile;
pub use snapshot::snapshot;
pub use snapshot_error::SnapshotError;
pub use snapshot_formatted::snapshot_formatted;
pub use snapshot_outcome::SnapshotOutcome;
pub use snapshot_target::SnapshotTarget;

#[cfg(test)]
mod candidate_size_tests;
#[cfg(test)]
mod cap_candidates_tests;
#[cfg(test)]
mod cap_mismatch_tests;
#[cfg(test)]
mod cap_refusals_tests;
#[cfg(test)]
mod capped_excludes_tests;
#[cfg(test)]
mod decide_formatter_result_tests;
#[cfg(test)]
mod decide_formatter_wait_tests;
#[cfg(test)]
mod decide_snapshot_tests;
#[cfg(test)]
mod empty_tree_tests;
#[cfg(test)]
mod formatter_argv_tests;
#[cfg(test)]
mod formatter_paths_tests;
#[cfg(test)]
mod literal_exclude_tests;
#[cfg(test)]
mod snapshot_excludes_tests;
#[cfg(test)]
mod snapshot_from_apply_tests;
#[cfg(test)]
mod snapshot_op_tests;
#[cfg(test)]
mod snapshot_parents_tests;
#[cfg(test)]
mod snapshot_ref_name_tests;
#[cfg(test)]
mod snapshot_trailers_tests;
