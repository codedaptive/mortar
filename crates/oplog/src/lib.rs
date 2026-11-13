//! oplog: see the Mortar mission (Layout).

pub mod apply;
pub mod history;
pub mod locks;
pub mod recover;
pub mod store;

pub use apply::{
    apply, check_ref_moves, close_after_update, find_done_record, new_op_id, op_id_from_parts, ref_updates,
    takeover_op, ApplyOutcome,
};
pub use history::{filter_history, history, HistoryFilter};
pub use locks::{
    acquire_lock, decide_lock_staleness, decode_lock_record, encode_lock_record, host_name,
    kill_status_alive, lock_file_name, lock_path, parse_host_name, release_lock, AcquiredLock,
    KillPidProbe, LockError, LockName, LockRecord, LockStaleness, PidProbe,
};
pub use recover::{decide_recovery, open_intents, recover, recovery_close, RecoveryDecision};
pub use store::{
    append_op_record, close_record, decode_op_log, decode_op_record, encode_op_record, intent_record,
    oplog_path, read_op_log, read_ref_value, ref_value_from_output, unix_now, Op, OpLog, OpPhase,
    OpRecord, OpResult, OplogError, RefMove,
};
