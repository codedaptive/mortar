//! What a save and a join hand the snapshot crate, built from the config by snapshot_settings_for.
//! A bare type; snapshot_settings_for_tests exercise it.

/// `formatter` goes to snapshot_formatted (None runs none); `excludes` is the `extra_excludes` and `cap_bytes`
/// the `size_cap` that snapshot_formatted takes, and snapshot::save_with_hooks too in save without a formatter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotSettings {
    pub formatter: Option<snapshot::Formatter>,
    pub excludes: Vec<String>,
    pub cap_bytes: u64,
}
