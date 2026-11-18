//! PACKAGE_PREFIXES: the package-manager prefixes under which install skips placement (spec 07 item 1).
//! No sibling test: a bare static with no behaviour (S-7). run_install reads it into InstallEnv.prefixes;
//! placement_skip_tests test placement_skip with prefixes of their own.

/// Homebrew's prefixes on Apple silicon and Intel: a binary under one is the package manager's to place.
pub const PACKAGE_PREFIXES: &[&str] = &["/opt/homebrew", "/usr/local"];
