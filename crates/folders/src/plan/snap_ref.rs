//! A request's snapshot ref name: `refs/mortar/snap/<zone>/<request>` (S-19).

use super::{check_name, PlanError, SNAP_REF_PREFIX};

/// SNAP_REF_PREFIX followed by `<zone>/<request>`, after both names pass check_name.
pub fn snap_ref(zone: &str, request: &str) -> Result<String, PlanError> {
    check_name("zone", zone)?;
    check_name("request", request)?;
    Ok(format!("{SNAP_REF_PREFIX}{zone}/{request}"))
}
