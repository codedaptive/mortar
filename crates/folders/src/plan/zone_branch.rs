//! A zone's branch name: `zone/<zone>` (S-17).

use super::{check_name, PlanError};

/// `zone/<zone>`, after the zone name passes check_name.
pub fn zone_branch(zone: &str) -> Result<String, PlanError> {
    check_name("zone", zone)?;
    Ok(format!("zone/{zone}"))
}
