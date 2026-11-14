//! A request's branch name: `req/<zone>/<request>`.

use super::{check_name, PlanError};

/// `req/<zone>/<request>`, after both names pass check_name.
pub fn request_branch(zone: &str, request: &str) -> Result<String, PlanError> {
    check_name("zone", zone)?;
    check_name("request", request)?;
    Ok(format!("req/{zone}/{request}"))
}
