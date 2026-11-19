//! An attempt's branch name: `try/<zone>/<request>/<k>`.

use super::{check_name, PlanError};

/// `try/<zone>/<request>/<k>`, a short name under refs/heads, after both names pass check_name. Attempts are
/// numbered from 1, so k == 0 is refused. The `try/` root keeps it apart from the request branch
/// `req/<zone>/<request>`, which git could not hold beside a ref nested under it.
pub fn attempt_branch(zone: &str, request: &str, k: u32) -> Result<String, PlanError> {
    check_name("zone", zone)?;
    check_name("request", request)?;
    if k == 0 {
        return Err(PlanError {
            reason: "attempt number 0; attempts are numbered from 1".to_string(),
        });
    }
    Ok(format!("try/{zone}/{request}/{k}"))
}
