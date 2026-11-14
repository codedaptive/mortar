//! cli: the shell of the `mortar` binary: arguments, the per-repository config, the folder and lock a verb
//! runs with, the op key, the verbs, and the report every verb prints.

pub mod args;
pub mod config;
pub mod context;
pub mod init;
pub mod join;
mod json;
pub mod lock;
pub mod opkey;
pub mod report;
pub mod save;
pub mod shell;

pub use args::{
    find_verb, parse_args, usage_report, LockNeed, ParsedArgs, UsageError, VerbKind, VerbSpec, VERBS,
};
pub use config::{
    config_path, decode_config, encode_config, protected_refs, read_config, snapshot_settings_for, write_config,
    Config, ConfigError, FormatterConfig, SnapshotSettings, ZoneConfig,
};
pub use context::{actor_for, context_report, read_folder_context, ContextError, FolderContext};
pub use init::{decide_init, init_report, parse_init_args, run_init, InitDecision, InitOptions};
pub use crate::join::{
    find_pending_move, join_refusal_report, join_report, join_save_subject, move_refusal_report, read_pending_move,
    run_join, save_part, JoinOutcome, PendingMove,
};
pub use lock::{lock_name_for, lock_report, locked_act};
pub use opkey::op_key;
pub use report::{failure_report, render_json, render_plain, ExitCode, OutValue, Report};
pub use save::{run_save, save_error_report, save_report, save_sentence};
pub use shell::{load_config, missing_config_report, run, ShellContext};
