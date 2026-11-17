//! cli: the shell of the `mortar` binary: arguments, the per-repository config, the folder and lock a verb
//! runs with, the op key, the verbs, and the report every verb prints.

pub mod after_publish;
pub mod args;
pub mod back_out;
pub mod backup;
pub mod bisect;
pub mod config;
pub mod context;
pub mod disk;
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
pub mod overlap;
pub mod publish;
pub mod report;
pub mod rereview;
pub mod resolve;
pub mod restore;
pub mod save;
pub mod shell;
pub mod squash;
pub mod status;
pub mod sweep;
pub mod undo;
pub mod undo_reason;
pub mod upkeep;
pub mod would_merge;

pub use after_publish::{
    after_publish_error_report, after_publish_failure, after_publish_report, bring_zone_folder, find_zone_old,
    parse_after_publish_args, read_named_zones, repeat_command, run_after_publish, zone_folder_lines, zone_move_lines,
    zone_values, AfterPublishArgs, FolderRefusal, NamedZone, ZoneFolderOutcome, ZoneOld,
};
pub use args::{
    find_verb, no_arguments_report, parse_args, usage_report, LockNeed, ParsedArgs, UsageError, VerbKind, VerbSpec,
    AFTER_PUBLISH_SPEC, BACKUP_SPEC, BISECT_SPEC, DISK_SPEC, OVERLAP_SPEC, RESTORE_BACKUP_SPEC, SNAPSHOT_SPEC,
    SWEEP_SPEC, UNDO_SPEC, VERBS,
};
pub use back_out::{back_out_report, run_back_out};
pub use backup::{
    backup_error_report, backup_report, parse_restore_backup_args, restore_backup_report, run_backup,
    run_restore_backup,
};
pub use bisect::{
    bisect_error_report, bisect_report, culprit_line, parse_bisect_args, probe_line, run_bisect, BisectArgs,
};
pub use config::{
    config_path, decode_config, encode_config, protected_refs, read_config, snapshot_settings_for, write_config,
    Config, ConfigError, FormatterConfig, SnapshotSettings, ZoneConfig,
};
pub use context::{actor_for, context_report, read_folder_context, ContextError, FolderContext};
pub use disk::{disk_report, run_disk};
pub use folder::{
    attempt_saved, attempts_made_report, attempts_saved, dropped_report, folder_error_report, folder_error_why,
    folder_kind_text, folder_made_report, folder_refusal, folder_removed_report, folder_setup_for, kept_report,
    parse_folder_args, pending_keep_move, request_folder_paths, request_folder_refusal, run_folder, run_folder_attempts,
    run_folder_drop, run_folder_keep, run_folder_remove, run_folder_request, run_folder_zone, zone_folder_paths,
    FolderCommand,
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
    snapshot_failure, AnswerPart, JoinOutcome, MoveCause, PendingMove, RequestRun,
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
pub use overlap::{overlap_error_report, overlap_report, parse_overlap_args, run_overlap};
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
pub use save::{
    run_save, run_snapshot, save_error_report, save_report, save_run_for, save_sentence, snapshot_error_report,
    snapshot_report, zone_folder_refusal,
};
pub use shell::{load_config, missing_config_report, run, ShellContext};
pub use squash::{parse_squash_args, run_squash, squash_report};
pub use status::{
    is_joined, missing_branch_report, move_stalled, open_requests_for, overlaps_with, read_branch_tips, read_overlaps,
    read_stalled_move, read_status, run_status, status_fields, status_lines, status_next, status_outcome, status_report,
    StatusState,
};
pub use sweep::{parse_sweep_args, run_sweep, sweep_build_failure_report, sweep_report, SweepArgs};
pub use undo::{
    already_undone_report, choose_undo, failed_undo_report, later_op_refusal, nothing_to_undo_report, own_refs,
    run_undo, undo_caller, undo_error_report, undo_refusal, undo_report, UndoChoice, UndoStep,
};
pub use undo_reason::{reversed_op, undo_reason};
pub use upkeep::{config_zones, folders_error_report, no_zones_report};
pub use would_merge::{parse_would_merge_args, run_would_merge, would_merge_report};
