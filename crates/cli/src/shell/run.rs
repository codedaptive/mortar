//! One command line, from the words to the report the binary prints.
//! No unit test: it runs git through read_folder_context, load_config and the verbs (S-5); the tests in src
//! (context/context_report_tests, args/usage_report_tests, actor_for_tests, repository_report_tests,
//! missing_config_report_tests) cover what it reports. Nagatha's integration group is to
//! cover, with the mortar binary as a child process in a throwaway repository: an unknown verb is exit 4 listing the verbs; `mr init` without
//! --actor is exit 4; with --json stdout holds one JSON object and nothing else, also for a usage error; an agent verb
//! outside a Mortar folder is refused, exit 1 (with `mr join`); a verb that needs the config is refused without
//! one, exit 1, next "mr init", and outside any git repository is refused, exit 1, outcome "not-a-folder" for an
//! agent verb (`mr join`) and "not-a-repository" for a harness verb that needs the config (with the first such
//! verb); `mortar guard-hook` runs in any repository with no folder and no --actor, and --actor with it is exit 4;
//! every report names its verb ("" when none was read).

use std::path::Path;

use gitio::Git;

use super::{load_config, ShellContext};
use crate::after_publish::run_after_publish;
use crate::args::{parse_args, usage_report, ParsedArgs, VerbKind};
use crate::back_out::run_back_out;
use crate::backup::{run_backup, run_restore_backup};
use crate::bisect::run_bisect;
use crate::config::protected_refs;
use crate::context::{actor_for, context_report, read_folder_context, repository_error, repository_report};
use crate::disk::run_disk;
use crate::folder::run_folder;
use crate::guard::run_guard;
use crate::guard_hook::run_guard_hook;
use crate::init::run_init;
use crate::join::run_join;
use crate::land::run_land;
use crate::log::run_log;
use crate::need::run_need;
use crate::overlap::run_overlap;
use crate::publish::run_publish;
use crate::report::{failure_report, Report};
use crate::rereview::run_rereview;
use crate::resolve::run_resolve;
use crate::restore::run_restore;
use crate::save::{run_save, run_snapshot};
use crate::squash::run_squash;
use crate::status::run_status;
use crate::sweep::run_sweep;
use crate::undo::run_undo;
use crate::would_merge::run_would_merge;

/// The report of `argv` (without the program name) run in `cwd`, and whether to render it as JSON. Agent verbs
/// read their folder first; every verb then gets its actor (a read-only verb acts as nobody, ""); verbs that need
/// the config load it and the protected list. The verb runs with the context built.
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
        VerbKind::Harness | VerbKind::Hook | VerbKind::ReadOnly => None,
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
        "after-publish" => run_after_publish(context, cwd),
        "back-out" => run_back_out(context, cwd),
        "backup" => run_backup(context, cwd),
        "bisect" => run_bisect(context, cwd),
        "disk" => run_disk(context, cwd),
        "folder" => run_folder(context, cwd),
        "guard" => run_guard(context, cwd),
        "guard-hook" => run_guard_hook(context, cwd),
        "init" => run_init(&context.args, cwd),
        "join" => run_join(context),
        "land" => run_land(context, cwd),
        "log" => run_log(context, cwd),
        "need" => run_need(context),
        "overlap" => run_overlap(context, cwd),
        "publish" => run_publish(context, cwd),
        "rereview" => run_rereview(context, cwd),
        "resolve" => run_resolve(context),
        "restore" => run_restore(context, cwd),
        "restore-backup" => run_restore_backup(context, cwd),
        "save" => run_save(context),
        "snapshot" => run_snapshot(context, cwd),
        "squash" => run_squash(context, cwd),
        "status" => run_status(context),
        "sweep" => run_sweep(context, cwd),
        "undo" => run_undo(context),
        "would-merge" => run_would_merge(context, cwd),
        other => failure_report(other, &format!("the verb {other} is in the verb table but has no runner")),
    }
}
