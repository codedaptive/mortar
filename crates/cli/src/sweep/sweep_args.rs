//! What `mr sweep` reads from its line.
//! A bare type (S-7); parse_sweep_args_tests exercises it.

/// `idle_days`: clear the build dirs of folders idle that many days, or None to clear none. `refs`: the caller's
/// extra full ref names to delete (S-53), in the order given, duplicates kept.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SweepArgs {
    pub idle_days: Option<u64>,
    pub refs: Vec<String>,
}
