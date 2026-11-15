//! cli: the shell of the `mortar` binary: arguments, the per-repository config, the folder and lock a verb
//! runs with, the op key, the verbs, and the report every verb prints.

pub mod args;
pub mod config;
pub mod context;
pub mod guard_hook;
pub mod init;
pub mod join;
mod json;
pub mod lock;
pub mod log;
pub mod need;
pub mod opkey;
pub mod report;
pub mod resolve;
pub mod restore;
pub mod save;
pub mod shell;
pub mod status;

pub use args::{
    find_verb, parse_args, usage_report, LockNeed, ParsedArgs, UsageError, VerbKind, VerbSpec, VERBS,
};
pub use config::{
    config_path, decode_config, encode_config, protected_refs, read_config, snapshot_settings_for, write_config,
    Config, ConfigError, FormatterConfig, SnapshotSettings, ZoneConfig,
};
pub use context::{actor_for, context_report, read_folder_context, ContextError, FolderContext};
pub use guard_hook::{guard_hook_from_args, guard_hook_report, run_guard_hook};
pub use init::{decide_init, init_report, parse_init_args, run_init, InitDecision, InitOptions};
pub use crate::join::{
    answer_part, attempt_refusal, find_pending_move, finish_pending_move, finished_move_part, join_refusal_report,
    join_report, join_save_subject, move_refusal_report, read_pending_move, request_run_for, run_join, save_part,
    snapshot_failure, AnswerPart, JoinOutcome, PendingMove, RequestRun,
};
pub use lock::{lock_name_for, lock_report, locked_act};
pub use log::{log_filters, log_lines, log_report, matching_records, run_log};
pub use need::{missing_path_report, need_error_report, need_report, read_need_path, run_need};
pub use opkey::op_key;
pub use report::{failure_report, prints_nothing, render_json, render_plain, ExitCode, OutValue, Report};
pub use resolve::{
    resolve_move_refusal_report, resolve_refusal_report, resolve_report, run_resolve, ResolveOutcome,
};
pub use restore::{
    folders_on_refs, folders_to_move, move_restored_folders, restore_error_report, restore_key, restore_lock,
    restore_move_refusal_report, restore_moves, restore_refusal, restore_report, run_restore, FolderOnRef,
};
pub use save::{run_save, save_error_report, save_report, save_sentence};
pub use shell::{load_config, missing_config_report, run, ShellContext};
pub use status::{
    is_joined, missing_branch_report, move_stalled, open_requests_for, overlaps_with, read_branch_tips, read_overlaps,
    read_stalled_move, read_status, run_status, status_fields, status_lines, status_next, status_outcome, status_report,
    StatusState,
};
