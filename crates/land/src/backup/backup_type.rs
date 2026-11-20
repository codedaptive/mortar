//! Backup: what one backup run did on the remote. A declaration only; backup_answer_tests and
//! the backup integration tests (backup_zit) exercise it.
//! Not named backup.rs: that file holds the function of the same name.

/// The local refnames this run backed up, each once.
/// `pushed`: set or deleted on the host's backup namespace by this run; `unchanged`: the backup already held it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Backup {
    pub pushed: Vec<String>,
    pub unchanged: Vec<String>,
}
