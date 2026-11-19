//! The PATH half of InstallFacts, read by read_path_facts.
//! A bare type; the install ZITs exercise it.

use std::path::PathBuf;

/// `first_mr`, `first_mortar`: the first of each on PATH as PATH names it, with whether it is Mortar.
/// `first_mr_at`: the index in path_entries of the entry `first_mr` was found in. `bin_at`: the index of the first
/// entry equal to `~/.local/bin`, None when it is not on PATH.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathFacts {
    pub first_mr: Option<(PathBuf, bool)>,
    pub first_mr_at: Option<usize>,
    pub first_mortar: Option<(PathBuf, bool)>,
    pub bin_at: Option<usize>,
}
