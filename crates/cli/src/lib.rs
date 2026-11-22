//! cli: the shell of the binary (named both `mortar` and `mr`): its entry, arguments, the per-repository config,
//! the folder and lock a verb runs with, the op key, the verbs, and the report every verb prints.
//! The crate exports `main_entry` only; everything else is crate-private.

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
mod help;
mod init;
mod install;
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
mod report;
mod rereview;
mod resolve;
mod restore;
mod save;
mod savings;
mod savings_record;
mod shell;
mod squash;
mod status;
mod sweep;
mod undo;
mod undo_reason;
mod upkeep;
mod version;
mod withdraw;
mod would_merge;
mod zone_folder_move;

pub use entry::main_entry;

pub(crate) use args::{usage_report, UsageError};
pub(crate) use report::{failure_report, prints_nothing, render_json, render_plain};
pub(crate) use shell::run;
