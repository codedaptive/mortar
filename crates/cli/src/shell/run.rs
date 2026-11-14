//! One command line, from the words to the report the binary prints.
//! No unit test: it runs git through read_folder_context, load_config and the verbs (S-5); usage_report_tests,
//! actor_for_tests, context_report_tests, repository_report_tests and missing_config_report_tests cover what
//! it reports. Nagatha's integration group is to cover, with the mortar binary as a child process in a
//! throwaway repository: an
//! unknown verb is exit 4 listing the verbs; `mortar init` without --actor is exit 4; with --json stdout holds
//! one JSON object and nothing else, also for a usage error; an agent verb outside a Mortar folder is refused,
//! exit 1 (with `mortar join`); a verb that needs the config is refused without one, exit 1, next "mortar init",
//! and outside any git repository is refused, exit 1, outcome "not-a-folder" for an agent verb (`mortar join`)
//! and "not-a-repository" for a harness verb that needs the config (with the first such verb); every report
//! names its verb ("" when none was read).

use std::path::Path;

use gitio::Git;

use super::{load_config, ShellContext};
use crate::args::{parse_args, usage_report, ParsedArgs, VerbKind};
use crate::config::protected_refs;
use crate::context::{actor_for, context_report, read_folder_context, repository_error, repository_report};
use crate::init::run_init;
use crate::join::run_join;
use crate::report::{failure_report, Report};
use crate::save::run_save;

/// The report of `argv` (without the program name) run in `cwd`, and whether to render it as JSON. Agent verbs
/// read their folder first; every verb then gets its actor; verbs that need the config load it and the
/// protected list. The verb runs with the context built.
pub fn run(argv: &[String], cwd: &Path) -> (Report, bool) {
    match parse_args(argv) {
        Err(error) => (usage_report(&error), error.json),
        Ok(args) => {
            let json = args.json;
            let report = shell_context(args, cwd).map_or_else(|refusal| refusal, |context| dispatch(&context, cwd));
            (report, json)
        }
    }
}

fn shell_context(args: ParsedArgs, cwd: &Path) -> Result<ShellContext, Report> {
    let verb = args.verb.name;
    let folder = match args.verb.kind {
        VerbKind::Agent => Some(read_folder_context(cwd).map_err(|error| context_report(verb, &error))?),
        VerbKind::Harness => None,
    };
    let actor = actor_for(args.verb, folder.as_ref().map(|f| &f.record), args.actor.as_deref())?;
    let config = match (args.verb.needs_config, &folder) {
        (false, _) => None,
        (true, Some(folder)) => Some(load_config(verb, &folder.common)?),
        (true, None) => {
            let common =
                Git::new(cwd).common_dir().map_err(|error| repository_report(verb, &repository_error(cwd, error)))?;
            Some(load_config(verb, &common)?)
        }
    };
    let protected = config.as_ref().map(protected_refs).unwrap_or_default();
    Ok(ShellContext { args, actor, folder, config, protected })
}

fn dispatch(context: &ShellContext, cwd: &Path) -> Report {
    match context.args.verb.name {
        "init" => run_init(&context.args, cwd),
        "join" => run_join(context),
        "save" => run_save(context),
        other => failure_report(other, &format!("the verb {other} is in the verb table but has no runner")),
    }
}
