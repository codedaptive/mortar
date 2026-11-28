//! HookedSquashError: why squash_with_hooks was refused. A declaration only, with no sibling test;
//! tests/squash_zit/hook_cases.rs exercises Refused and Unreadable, hooked_squash_error_from_tests the Squash arm.

use crate::squash::SquashError;

/// A squash_with_hooks refusal. Debug only: SquashError is neither Clone nor Eq.
#[derive(Debug)]
pub enum HookedSquashError {
    /// Every refusal squash_with_message makes, as it makes it.
    Squash(SquashError),
    /// pre-commit or commit-msg refused the squash commit: nothing written, no op.
    Refused(gitio::HookRefusal),
    /// The message the hooks left does not read back as this squash: `part` is Mortar-Change-Id when any
    /// Mortar-Change-Id value in the hooked message's trailers is malformed; else the first written Mortar trailer
    /// (in written order: Mortar-Change-Id, Mortar-Zone, Mortar-Squash-Of) whose last value there is missing or
    /// differs; else "requests". `message` is the hooked text. Nothing written, no op.
    Unreadable { part: String, message: String },
}
