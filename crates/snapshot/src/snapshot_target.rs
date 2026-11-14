//! SnapshotTarget: which zone, request and attempt a snapshot belongs to, and who takes it. A declaration only;
//! snapshot_ref_name_tests exercises its zone, request and attempt; snapshot_trailers_tests, save_trailers_tests
//! and save_change_id_tests its zone and request.

/// The caller supplies it: Mortar's folder record (S-18) is read above this crate. `zone`, `request` and `attempt`
/// name the snapshot ref; `zone` and `request` name its trailers; `actor` is the op log's actor for the ref move.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapshotTarget {
    pub zone: String,
    pub request: String,
    /// None for the request's own folder, Some(k) for its attempt folder k, k from 1 (S-69, S-83); it chooses the
    /// snapshot ref. snapshot_ref_name refuses Some(0) with BadTarget.
    pub attempt: Option<u32>,
    pub actor: String,
}
