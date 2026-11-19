//! The body of run: one parsed command line taken to its report, with the version and folder reads given.
//! Unit tests: shell/run_in_tests, with both reads faked. The validation tests (to write) that cover the whole line
//! through the mortar binary are listed in shell/run.rs.

use std::path::Path;

use gitio::Git;

use super::{git_floor, load_config, ShellContext, ShellIo};
use crate::after_publish::run_after_publish;
use crate::args::{parse_args, usage_report, ParsedArgs, VerbKind};
use crate::back_out::run_back_out;
use crate::backup::{run_backup, run_restore_backup};
use crate::bisect::run_bisect;
use crate::config::protected_refs;
use crate::context::{actor_for, context_report, repository_error, repository_report, ContextError, FolderContext};
use crate::disk::run_disk;
use crate::folder::run_folder;
use crate::guard::run_guard;
use crate::guard_hook::run_guard_hook;
use crate::init::run_init;
use crate::install::{run_install, run_uninstall};
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

/// The report of `argv` (without the program name) run in `cwd`, and whether to render it as JSON. The line is
/// parsed; then git_floor calls `io.read_version` once (for every verb but the exempt ones) and stops with its
/// report, before anything else runs. Agent verbs read their folder next through `io.read_folder`, and no other
/// kind reads one (a setup verb, install or uninstall, reads no folder or config); every verb then gets its actor
/// (a read-only or setup verb acts as nobody, ""); verbs that need the config load it and the protected list. The
/// verb runs with the context built.
pub(crate) fn run_in(argv: &[String], cwd: &Path, io: &ShellIo) -> (Report, bool) {
    match parse_args(argv) {
        Err(error) => (usage_report(&error), error.json),
        Ok(args) => {
            let json = args.json;
            let report = git_floor(args.verb.name, io.read_version)
                .and_then(|()| shell_context(args, cwd, io.read_folder))
                .map_or_else(|refusal| refusal, |context| dispatch(&context, cwd));
            (report, json)
        }
    }
}

fn shell_context(
    args: ParsedArgs,
    cwd: &Path,
    read_folder: &dyn Fn(&Path) -> Result<FolderContext, ContextError>,
) -> Result<ShellContext, Report> {
    let verb = args.verb.name;
    let folder = match args.verb.kind {
        VerbKind::Agent => Some(read_folder(cwd).map_err(|error| context_report(verb, &error))?),
        VerbKind::Harness | VerbKind::Hook | VerbKind::ReadOnly | VerbKind::Setup => None,
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

// The verb table's help entry never reaches dispatch: main_entry answers every help line before run.
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
        "install" => run_install(context, cwd),
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
        "uninstall" => run_uninstall(context),
        "would-merge" => run_would_merge(context, cwd),
        other => failure_report(other, &format!("the verb {other} is in the verb table but has no runner")),
    }
}
