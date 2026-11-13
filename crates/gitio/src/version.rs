//! Version: a git release number.

/// major.minor.patch; the field order makes the derived ordering compare major, then minor, then patch.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}
