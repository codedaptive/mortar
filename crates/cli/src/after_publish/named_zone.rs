//! NamedZone: one zone named on the after-publish line or by overlap, as read_named_zones reads it (before land's
//! call, for after-publish). A bare type (S-7); run_after_publish and run_overlap read it, and Nagatha's RITs
//! listed there exercise it.

use std::path::PathBuf;

use join::OpenRequest;

/// The zone's name, its zone folder (None when it has none), and its open requests as land's after_publish takes
/// them (S-45).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedZone {
    pub zone: String,
    pub folder: Option<PathBuf>,
    pub requests: Vec<OpenRequest>,
}
