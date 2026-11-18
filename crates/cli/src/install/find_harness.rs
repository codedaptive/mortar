//! find_harness: One of HARNESSES by its exact name.

use super::{HarnessSpec, HARNESSES};

/// The harness named `name` exactly, or None.
pub fn find_harness(name: &str) -> Option<&'static HarnessSpec> {
    HARNESSES.iter().find(|harness| harness.name == name)
}
