//! Lock files: names, paths, record text, the decisions on liveness, host and staleness, and taking,
//! taking over and releasing a lock on disk.

mod acquire_lock;
mod acquired_lock;
mod decide_lock_staleness;
mod decode_lock_record;
mod encode_lock_record;
mod host_name;
mod kill_pid_probe;
mod kill_status_alive;
mod lock_error;
mod lock_error_display;
mod lock_error_removed_guards;
mod lock_error_source;
mod lock_error_with_removed_guards;
mod lock_file_name;
mod lock_name;
mod lock_path;
mod lock_record;
mod lock_staleness;
mod parse_host_name;
mod pid_probe;
mod read_lock_file;
mod release_lock;
mod take_over_stale_lock;
mod takeover_guard_path;
mod write_new_lock_file;

#[cfg(test)]
mod decide_lock_staleness_tests;
#[cfg(test)]
mod decode_lock_record_tests;
#[cfg(test)]
mod encode_lock_record_tests;
#[cfg(test)]
mod kill_status_alive_tests;
#[cfg(test)]
mod lock_error_display_tests;
#[cfg(test)]
mod lock_error_removed_guards_tests;
#[cfg(test)]
mod lock_error_source_tests;
#[cfg(test)]
mod lock_error_with_removed_guards_tests;
#[cfg(test)]
mod lock_file_name_tests;
#[cfg(test)]
mod lock_path_tests;
#[cfg(test)]
mod parse_host_name_tests;
#[cfg(test)]
mod takeover_guard_path_tests;

pub use acquire_lock::acquire_lock;
pub use acquired_lock::AcquiredLock;
pub use decode_lock_record::decode_lock_record;
pub use host_name::host_name;
pub use kill_pid_probe::KillPidProbe;
pub use lock_error::LockError;
pub use lock_name::LockName;
pub use lock_path::lock_path;
pub use lock_record::LockRecord;
pub use pid_probe::PidProbe;
pub use release_lock::release_lock;

pub(crate) use decide_lock_staleness::decide_lock_staleness;
pub(crate) use encode_lock_record::encode_lock_record;
pub(crate) use kill_status_alive::kill_status_alive;
pub(crate) use lock_file_name::lock_file_name;
pub(crate) use lock_staleness::LockStaleness;
pub(crate) use parse_host_name::parse_host_name;
pub(crate) use takeover_guard_path::takeover_guard_path;
