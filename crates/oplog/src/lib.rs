//! oplog: the operation log, undo and restore, locks, change IDs, and repeat-safe operation keys.

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
    apply, apply_with_values, find_done_record, new_op_id, op_git, open_remote_intent, takeover_op, with_lock,
    ApplyOutcome, HeldLockError, RemoteIntentOutcome, ValuedOutcome, MORTAR_OP_ENV,
};
pub use change_id::{new_change_id, ChangeId, ChangeIdError};
pub use history::{history, history_matches, HistoryFilter};
pub use locks::{
    acquire_lock, decode_lock_record, host_name, lock_path, release_lock, AcquiredLock, KillPidProbe, LockError,
    LockName, LockRecord, PidProbe,
};
pub use recover::{
    close_intent, decide_recovery, open_intent, open_intents, open_marked_intents, recover, recovery_close_valued,
    IntentClose, RecoveryDecision,
};
pub use store::{
    append_op_record, append_valued_record, close_record, encode_marked_record, intent_record, oplog_path,
    parse_json, parse_json_string, read_marked_log, read_op_log, read_ref_value, read_valued_log, unix_now,
    write_json_string, JsonError, JsonValue, MarkedLog, MarkedRecord, Op, OpLog, OpPhase, OpRecord, OpResult,
    OplogError, RefMove, ValuedLog, ValuedRecord,
};
pub use trailers::{
    mortar_trailer_pairs, pick_mortar_trailers, read_mortar_trailers, write_mortar_trailers, MortarTrailerError,
    MortarTrailers, MORTAR_CHANGE_ID_KEY, MORTAR_SQUASH_OF_KEY, MORTAR_ZONE_KEY,
};
pub use protect::{apply_protected, ProtectedRef};
pub use undo::{
    check_restore_ref_names, find_undo_target, plan_restore, restore, undo, OpCaller,
};
