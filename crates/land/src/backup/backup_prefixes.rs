//! BACKUP_PREFIXES: the local ref namespaces backup mirrors to the remote. A constant only;
//! local_ref_tests exercise it through local_ref, which maps a backup ref back only under one of these.

/// The in-flight refs: zone and request branches, snapshots and squashes. Nothing else is backed up or restored,
/// so a stray backup ref can never become `refs/heads/main` or any other local ref.
pub(crate) const BACKUP_PREFIXES: [&str; 4] =
    ["refs/heads/zone/", "refs/heads/req/", "refs/mortar/snap/", "refs/mortar/squash/"];
