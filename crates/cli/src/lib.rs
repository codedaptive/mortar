//! cli: the shell of the `mortar` binary: arguments, the per-repository config, the folder and lock a verb
//! runs with, the op key, the verbs, and the report every verb prints.

pub mod args;
pub mod config;
pub mod context;
pub mod init;
mod json;
pub mod lock;
pub mod opkey;
pub mod report;
pub mod shell;

pub use args::{
    find_verb, parse_args, usage_report, LockNeed, ParsedArgs, UsageError, VerbKind, VerbSpec, VERBS,
};
pub use config::{
    config_path, decode_config, encode_config, protected_refs, read_config, write_config, Config, ConfigError,
    FormatterConfig, ZoneConfig,
};
pub use context::{actor_for, context_report, read_folder_context, ContextError, FolderContext};
pub use init::{decide_init, init_report, parse_init_args, run_init, InitDecision, InitOptions};
pub use lock::{lock_name_for, lock_report, locked_act};
pub use opkey::op_key;
pub use report::{failure_report, render_json, render_plain, ExitCode, OutValue, Report};
pub use shell::{load_config, missing_config_report, run, ShellContext};
