//! cli: the shell of the `mortar` binary: arguments, the per-repository config, the folder and lock a verb
//! runs with, the op key, the verbs, and the report every verb prints.

pub mod args;
pub mod back_out;
pub mod config;
pub mod context;
pub mod folder;
pub mod guard;
pub mod guard_hook;
pub mod init;
pub mod integration_folder;
pub mod join;
mod json;
pub mod land;
pub mod landing;
pub mod lock;
pub mod log;
pub mod need;
pub mod opkey;
pub mod publish;
pub mod report;
pub mod rereview;
pub mod resolve;
pub mod restore;
pub mod save;
pub mod shell;
pub mod squash;
pub mod status;
pub mod would_merge;

pub use args::{
    find_verb, no_arguments_report, parse_args, usage_report, LockNeed, ParsedArgs, UsageError, VerbKind, VerbSpec,
    VERBS,
};
pub use back_out::{back_out_report, run_back_out};
pub use config::{
    config_path, decode_config, encode_config, protected_refs, read_config, snapshot_settings_for, write_config,
    Config, ConfigError, FormatterConfig, SnapshotSettings, ZoneConfig,
};
pub use context::{actor_for, context_report, read_folder_context, ContextError, FolderContext};
pub use folder::{
    attempt_saved, attempts_made_report, attempts_saved, dropped_report, folder_error_report, folder_error_why,
    folder_made_report, folder_refusal, folder_removed_report, folder_setup_for, kept_report, parse_folder_args,
    pending_keep_move, request_folder_paths, request_folder_refusal, run_folder, run_folder_attempts, run_folder_drop,
    run_folder_keep, run_folder_remove, run_folder_request, run_folder_zone, zone_folder_paths, FolderCommand,
};
pub use guard::{
    foreign_hook_report, guard_error_report, guard_fields, guard_in_place, guard_installed_report, guard_refs_for,
    guard_state_report, parse_guard_args, run_guard, run_guard_install, run_guard_status, GuardCommand,
};
pub use guard_hook::{guard_hook_from_args, guard_hook_report, run_guard_hook};
pub use init::{decide_init, init_report, parse_init_args, run_init, InitDecision, InitOptions};
pub use integration_folder::{
    finish_stopped_move, folder_move_refusal_report, integration_folder, integration_folder_line,
    integration_move_stopped, last_integration_old, move_integration_folder, note_finished_move, FinishedMove,
    MoveBlock,
};
pub use crate::join::{
    answer_part, attempt_refusal, find_pending_move, finish_pending_move, finished_move_part, join_refusal_report,
    join_report, join_save_subject, move_refusal_report, read_pending_move, request_run_for, run_join, save_part,
    snapshot_failure, AnswerPart, JoinOutcome, PendingMove, RequestRun,
};
pub use crate::land::{land_report, parse_land_args, run_land};
pub use landing::{
    land_error_report, landing_state_error_report, pick_squash_ref, publish_error_report, resolve_squash,
    squash_error_report, squash_pick_report, SquashName, SquashPick, CHANGE_CONFLICT_NEXT,
};
pub use lock::{lock_name_for, lock_report, locked_act};
pub use log::{log_filters, log_lines, log_report, matching_records, run_log};
pub use need::{missing_path_report, need_error_report, need_report, read_need_path, run_need};
pub use opkey::op_key;
pub use publish::{publish_report, run_publish};
pub use report::{failure_report, prints_nothing, render_json, render_plain, ExitCode, OutValue, Report};
pub use rereview::{parse_rereview_args, rereview_error_report, rereview_report, run_rereview};
pub use resolve::{
    resolve_move_refusal_report, resolve_refusal_report, resolve_report, run_resolve, ResolveOutcome,
};
pub use restore::{
    folders_on_refs, folders_to_move, move_restored_folders, restore_error_report, restore_key, restore_lock,
    restore_move_refusal_report, restore_moves, restore_refusal, restore_report, run_restore, FolderOnRef,
};
pub use save::{run_save, save_error_report, save_report, save_sentence};
pub use shell::{load_config, missing_config_report, run, ShellContext};
pub use squash::{parse_squash_args, run_squash, squash_report};
pub use status::{
    is_joined, missing_branch_report, move_stalled, open_requests_for, overlaps_with, read_branch_tips, read_overlaps,
    read_stalled_move, read_status, run_status, status_fields, status_lines, status_next, status_outcome, status_report,
    StatusState,
};
pub use would_merge::{parse_would_merge_args, run_would_merge, would_merge_report};
