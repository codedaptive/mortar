//! run_guard_hook: `mortar guard-hook <name>`, the verb the guard's git hooks run (SPEC-land 06).
//! No unit test (S-104): every decision that reads the repository runs land, oplog and gitio for real, so its tests
//! are RITs (S-101), owed to Nagatha. The unreadable-input refusal too comes after git and guard.json are read;
//! guard_hook_report_tests covers how it renders and the RIT case below covers its being given.
//! guard_hook_from_args_tests and guard_hook_report_tests cover the words it reads and the reports it gives. Her
//! integration group is to cover,
//! with the mortar binary installed as the hook (land::guard_install; mortar guard install once built, S-112) in a
//! throwaway repository, git run with
//! GIT_CONFIG_GLOBAL=/dev/null and GIT_CONFIG_NOSYSTEM=1:
//! - an allowed move prints nothing on stdout or stderr, exit 0, also with --json (S-111);
//! - hook input on stdin that is not UTF-8 is refused (exit 1), with nothing moved;
//! - update-ref, commit and push against integration and origin are refused, with nothing moved;
//! - MORTAR_OP naming no open intent is refused;
//! - apply_protected and publish's push pass;
//! - unguarded refs move freely;
//! - a push by the guarded remote's URL is refused like a push by its name, a push to another remote is allowed,
//!   and with the remote's URLs unreadable a push under another name is refused (S-100);
//! - with the hooks present and guard.json missing, a move is refused;
//! - a guard.json that is not a guard config is refused, naming the file.

use std::io::Read;
use std::path::Path;

use gitio::Git;
use land::{guard_decide, read_guard_config, GuardError, GuardHook, GuardVerdict, LandRefs};
use oplog::{open_intent, RefMove, MORTAR_OP_ENV};

use super::{guard_hook_from_args, guard_hook_report};
use crate::args::{usage_report, UsageError};
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

const VERB: &str = "guard-hook";
const INSTALLER: &str = "the harness that installs the guard";

/// Reads guard.json under the common dir of the repository at `cwd`; then the hook's words (after guard.json, because a
/// pre-push needs guard.json's remote for its push URLs); then the hook's input from stdin; then the moves of the
/// open intent MORTAR_OP names (None when it is unset or not unicode), and reports land::guard_decide's verdict
/// (an allow's report is exit Done with nothing in it, and the binary prints none of it: S-111).
/// A guard.json that is missing or is not a guard config, and input that cannot be read, are refused (exit 1): the
/// guard cannot check the move, so it fails closed. Words it cannot read are usage (exit 4); git or the op log
/// failing is failure_report (exit 5). Takes no lock and moves nothing.
pub fn run_guard_hook(context: &ShellContext, cwd: &Path) -> Report {
    decide(context, cwd).unwrap_or_else(|report| report)
}

fn decide(context: &ShellContext, cwd: &Path) -> Result<Report, Report> {
    let git = Git::new(cwd);
    let common = git.common_dir().map_err(|error| failure_report(VERB, &error.to_string()))?;
    let refs = read_refs(&git, &common)?;
    Ok(guard_hook_report(&guard_decide(
        &hook(context, &git, &refs)?,
        &read_input()?,
        &refs,
        open_moves(&common)?.as_deref(),
    )))
}

/// The hook's words; words it cannot read are usage (exit 4), with the verb's usage line as next.
fn hook(context: &ShellContext, git: &Git, refs: &LandRefs) -> Result<GuardHook, Report> {
    guard_hook_from_args(&context.args.rest, || git.remote_push_urls(&refs.remote).ok()).map_err(|reason| Report {
        next: context.args.verb.usage.to_string(),
        ..usage_report(&UsageError { verb: Some(VERB.to_string()), reason, json: context.args.json })
    })
}

/// The moves of the open intent MORTAR_OP names; None when it is unset or not unicode.
fn open_moves(common: &Path) -> Result<Option<Vec<RefMove>>, Report> {
    match std::env::var(MORTAR_OP_ENV) {
        Ok(op) => open_intent(common, &op).map_err(|error| failure_report(VERB, &error.to_string())),
        Err(_) => Ok(None),
    }
}

/// All of stdin as UTF-8 text; a read error or bytes that are not UTF-8 are refused.
fn read_input() -> Result<String, Report> {
    let mut bytes = Vec::new();
    std::io::stdin()
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())
        .and_then(|_| String::from_utf8(bytes).map_err(|error| format!("it is not UTF-8: {error}")))
        .map_err(|why| {
            refuse(
                format!("the guard cannot read the hook's input ({why}), so it fails closed"),
                "repeat the git command; if the guard still cannot read its input, ask the harness that installs \
                 the guard to install it again",
            )
        })
}

/// The branches guard.json protects. Missing while its hooks run, or not a guard config: refused. Any other
/// failure to read it: failure_report.
fn read_refs(git: &Git, common: &Path) -> Result<LandRefs, Report> {
    match read_guard_config(git) {
        Ok(Some(refs)) => Ok(refs),
        Ok(None) => Err(refuse(
            format!(
                "the guard's hooks ran but its config guard.json is missing from {}, so no move can be checked and \
                 the guard fails closed",
                common.join("mortar").display()
            ),
            "ask the harness that installs the guard to install it again",
        )),
        Err(error @ GuardError::BadConfig { .. }) => Err(refuse(
            error.to_string(),
            "mend or remove guard.json by hand, then ask the harness that installs the guard to install it again",
        )),
        Err(error) => Err(failure_report(VERB, &error.to_string())),
    }
}

/// A refusal the cli gives before guard_decide runs, rendered as guard_hook_report renders the guard's own.
fn refuse(why: String, next: &str) -> Report {
    guard_hook_report(&GuardVerdict::Refuse { why, owner: INSTALLER.to_string(), next: next.to_string() })
}
