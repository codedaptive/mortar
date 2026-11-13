//! Which lock: one per zone for joins, one for landing.
//! A bare type; lock_file_name_tests and lock_path_tests exercise it.

/// A typed lock name. `Join` carries the zone; its spelling is checked by `lock_file_name`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LockName {
    Join(String),
    Land,
}
