//! What apply makes of git check-ref-format's answer for one move's ref name.

use crate::store::OplogError;

/// Ok when git found `refname` well formed; otherwise `BadMove` naming it.
pub(crate) fn ref_format_verdict(refname: &str, well_formed: bool) -> Result<(), OplogError> {
    if well_formed {
        return Ok(());
    }
    Err(OplogError::BadMove {
        refname: refname.to_string(),
        reason: "is not a valid ref name (git check-ref-format refused it)".to_string(),
    })
}
