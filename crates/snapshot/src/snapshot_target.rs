//! SnapshotTarget: which zone and request a snapshot belongs to, and who takes it. A declaration only;
//! snapshot_ref_name_tests and snapshot_trailers_tests exercise it.

/// The caller supplies it: Mortar's folder record (S-18) is read above this crate. `zone` and `request` name the
/// snapshot ref and its trailers; `actor` is the op log's actor for the ref move.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapshotTarget {
    pub zone: String,
    pub request: String,
    pub actor: String,
}
