//! cli: the shell of the binary (named both `mortar` and `mr`): its entry, arguments, the per-repository config,
//! the folder and lock a verb runs with, the op key, the verbs, and the report every verb prints.
//! The crate exports `main_entry`, the items the integration tests in crates/cli/tests import, and the types those
//! items' signatures name.

mod after_publish;
mod args;
mod back_out;
mod backup;
mod bisect;
mod config;
mod context;
mod disk;
mod entry;
mod folder;
mod guard;
mod guard_hook;
mod init;
mod integration_folder;
mod join;
mod json;
mod land;
mod landing;
mod lock;
mod log;
mod need;
mod opkey;
mod overlap;
mod path;
mod publish;
pub mod report;
mod rereview;
mod resolve;
mod restore;
mod save;
mod shell;
mod squash;
mod status;
mod sweep;
pub mod undo;
mod undo_reason;
mod upkeep;
mod would_merge;

pub use args::{find_verb, LockNeed, ParsedArgs, VerbKind, VerbSpec};
pub use config::{Config, FormatterConfig, SnapshotSettings, ZoneConfig};
pub use context::{context_report, ContextError, FolderContext};
pub use entry::main_entry;
pub use crate::join::{
    join_refusal_report, request_run_for, snapshot_failure, MoveCause, PendingMove, RequestRun,
};
pub use lock::lock_report;
pub use report::{render_json, render_plain, ExitCode, OutValue, Report};
pub use resolve::resolve_refusal_report;
pub use save::{save_error_report, save_sentence};
pub use shell::ShellContext;
pub use status::{status_report, StatusState};

pub(crate) use args::{usage_report, UsageError};
pub(crate) use report::{failure_report, prints_nothing};
pub(crate) use shell::run;
