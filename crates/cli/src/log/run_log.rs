//! run_log: `mr log [--ref <r>] [--actor <a>] [--key <k>]`, the read-only verb that lists the op log's records.
//! It runs git for the common dir and reads the op log file; log_filters_tests,
//! matching_records_tests, log_lines_tests and log_report_tests cover its decisions, run_log_tests its usage refusals
//! before any read, and the RIT crates/cli/tests/cli_rit/run_log_tests.rs, for one op in a real log, the plain
//! listing with no "records" lines and the --json "records" line, for a line with no values or remote mark and for
//! one carrying both. The repo integration tests are to cover, with the mortar binary as a child process in a
//! throwaway repository with ops in its log (tests running real git set GIT_CONFIG_GLOBAL=/dev/null and
//! GIT_CONFIG_NOSYSTEM=1):
//! - a real log read with each filter (--ref, --actor, --key) lists the matching ops, and with two filters their AND;
//! - with no filter it lists every op; a filter nothing matches is exit 0, outcome empty;
//! - with several ops in the log, --json "records" are the log's own lines for those ops, byte for byte;
//! - log writes nothing: refs, the op log and the lock folder are unchanged after it;
//! - run outside any git repository it is refused, exit 1, outcome not-a-repository.

use std::path::Path;

use gitio::Git;
use oplog::{read_marked_log, OpRecord};

use super::{encoded_records, log_filters, log_report, matching_records};
use crate::args::{usage_report, UsageError};
use crate::context::{repository_error, repository_report};
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

/// Reads the filters first (log_filters: a bad line is log's usage, exit 4, before any read); then the repository's
/// common dir from `cwd` (no repository: repository_report) and its op log (unreadable: failure_report), and
/// reports the records every filter keeps (matching_records) through log_report, with --json each record's own log
/// line, values and remote mark included (encoded_records). No lock, no op, writes nothing.
pub fn run_log(context: &ShellContext, cwd: &Path) -> Report {
    let args = &context.args;
    let filters = match log_filters(&args.rest, args.key.as_deref(), args.actor.as_deref()) {
        Ok(filters) => filters,
        Err(reason) => return usage_report(&UsageError { verb: Some("log".to_string()), reason, json: args.json }),
    };
    let common = match Git::new(cwd).common_dir() {
        Ok(common) => common,
        Err(error) => return repository_report("log", &repository_error(cwd, error)),
    };
    match read_marked_log(&common) {
        Ok(log) => {
            let marked = matching_records(&log.records, &filters);
            let records: Vec<&OpRecord> = marked.iter().map(|m| &m.record).collect();
            log_report(&records, encoded_records(&marked, args.json))
        }
        Err(error) => failure_report("log", &error.to_string()),
    }
}
