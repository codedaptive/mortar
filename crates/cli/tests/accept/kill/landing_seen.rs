//! The landing in flight as `<common>/mortar/landing.json` holds it. Data only: read_landing makes it, and
//! read_landing_tests.rs covers how.

/// The squash being landed (change ID, zone, commit), the origin tip when the landing started, and the commit the
/// landing moves integration to.
#[derive(Debug, Clone, PartialEq)]
pub struct LandingSeen {
    pub change_id: String,
    pub zone: String,
    pub squash: String,
    pub origin_at_start: String,
    pub integration: String,
}
