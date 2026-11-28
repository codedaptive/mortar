//! HookedSquashError from a SquashError, so `?` carries squash_steps' refusals into squash_with_hooks.
//! hooked_squash_error_from_tests covers it.

use crate::squash::{HookedSquashError, SquashError};

impl From<SquashError> for HookedSquashError {
    /// The refusal unchanged, as Squash.
    fn from(error: SquashError) -> Self {
        HookedSquashError::Squash(error)
    }
}
