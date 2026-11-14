//! oplog: see the Mortar mission (Layout).

pub mod apply;
pub mod change_id;
pub mod history;
pub mod locks;
pub mod protect;
pub mod recover;
pub mod store;
pub mod trailers;
pub mod undo;

pub use apply::{
    apply, check_ref_moves, check_ref_names, close_after_update, find_done_record, guard_removal_op,
    new_op_id, op_id_from_parts, ref_format_verdict, ref_updates, takeover_op, ApplyOutcome,
};
pub use change_id::{new_change_id, ChangeId, ChangeIdError};
pub use history::{filter_history, history, HistoryFilter};
pub use locks::{
    acquire_lock, decide_lock_staleness, decode_lock_record, encode_lock_record, host_name,
    kill_status_alive, lock_file_name, lock_path, parse_host_name, release_lock, AcquiredLock,
    KillPidProbe, LockError, LockName, LockRecord, LockStaleness, PidProbe,
};
pub use recover::{decide_recovery, open_intents, recover, recovery_close, RecoveryDecision};
pub use store::{
    append_op_record, close_record, decode_op_log, decode_op_record, encode_op_record, intent_record,
    oplog_path, read_op_log, read_ref_value, unix_now, Op, OpLog, OpPhase, OpRecord, OpResult,
    OplogError, RefMove,
};
pub use trailers::{
    carry_mortar_trailers, mortar_trailer_pairs, pick_mortar_trailers, read_mortar_trailers,
    write_mortar_trailers, MortarTrailerError, MortarTrailers, MORTAR_CHANGE_ID_KEY, MORTAR_REQUEST_KEY,
    MORTAR_ZONE_KEY,
};
pub use protect::{apply_protected, check_protected_moves, protected_move_verdict, ProtectedRef};
pub use undo::{
    caller_op, find_undo_target, last_mover, plan_restore, plan_undo, restore, undo, value_as_of_op, OpCaller,
};
