//! run_overlap: `mr overlap <zone>`, which open requests of the zone change the same paths, before anyone joins.
//! Takes no lock and writes nothing.
//! No sibling test: it runs git and reads the folders' records through read_named_zones and join's overlap;
//! parse_overlap_args_tests, overlap_report_tests and overlap_error_report_tests cover its decisions.
//! The integration tests crates/cli/tests/across_kits/snapshot_overlap_disk_sweep/{overlap,guards}.rs cover, with the
//! mortar binary as a child process in request folders made by the landing verbs in a throwaway repository:
//! - overlap with two requests touching one file lists the pair and that path, exit 0, outcome overlaps;
//! - requests touching different files give no pair, exit 0, outcome no-overlap;
//! - a request rebased onto a zone that moved shows no overlap on paths only the zone changed (its base is
//!   its fork point from the zone);
//! - overlap of a zone not in the config, no zone, or a second word, exit 4; without a config, exit 1;
//! - a request whose zone branch is missing, exit 5 under the verb overlap, naming the ref;
//! - overlap moves no ref and writes nothing.

use std::path::Path;

use gitio::Git;

use super::{overlap_error_report, overlap_report, parse_overlap_args};
use crate::after_publish::read_named_zones;
use crate::args::{usage_report, UsageError};
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

const VERB: &str = "overlap";

/// The config, then the zone read by parse_overlap_args (an Err is overlap's usage report, exit 4, before the verb
/// runs git). Then the zone's open requests from the folder records, each based at its fork point from the zone
/// (read_named_zones; its failure report is relabelled to the verb overlap), and join's overlap over
/// them (a failure is overlap_error_report); reported by overlap_report.
pub fn run_overlap(context: &ShellContext, cwd: &Path) -> Report {
    let args = &context.args;
    let Some(config) = &context.config else {
        return failure_report(VERB, "no config for a verb that needs it; the shell loads it before overlap runs");
    };
    let zone = match parse_overlap_args(&args.rest, args.dash_dash, config) {
        Ok(zone) => zone,
        Err(reason) => {
            return usage_report(&UsageError { verb: Some(VERB.to_string()), reason, json: args.json });
        }
    };
    let git = Git::new(cwd);
    let named = match read_named_zones(&git, std::slice::from_ref(&zone)) {
        Ok(named) => named,
        Err(mut report) => {
            report.verb = VERB.to_string();
            return report;
        }
    };
    let requests = named.into_iter().flat_map(|named| named.requests).collect::<Vec<_>>();
    match ::join::overlap(&git, &requests) {
        Ok(pairs) => overlap_report(&zone, &pairs),
        Err(error) => overlap_error_report(&error),
    }
}
