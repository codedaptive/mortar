//! oplog: see the Mortar mission (Layout).

pub mod locks;

pub use locks::{
    acquire_lock, decide_lock_staleness, decode_lock_record, encode_lock_record, host_name,
    kill_status_alive, lock_file_name, lock_path, parse_host_name, release_lock, AcquiredLock,
    KillPidProbe, LockError, LockName, LockRecord, LockStaleness, PidProbe,
};
