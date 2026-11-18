//! Everything install reads before it decides, so the plan is a function of data (spec 07, "Tests": the plan as
//! data). A bare type (S-7); the planning tests exercise it.

use std::path::PathBuf;

use super::{BinaryAt, HarnessSpec, LinkAt, RecordItem, SkillAt};

/// `this`: this binary's canonical path; `this_hash`: content_hash of its bytes. `binary_at`, `link_at`: what stands
/// at `~/.local/bin/mortar` and `~/.local/bin/mr`. `first_mr`, `first_mortar`: the first of each on PATH as PATH
/// names it, with whether it is Mortar. `first_mr_at`: the index in path_entries of the entry `first_mr` was found
/// in. `bin_at`: the index of the first entry equal to `~/.local/bin`, None when it is not on PATH. `exe_dir`: the
/// folder of the running binary's path as given (not canonical). `detected`: the harnesses whose folder is in HOME.
/// `skills`: one per detected harness, its skill path at the chosen location and what stands there. `record`: the
/// earlier install's record, empty when there is none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallFacts {
    pub this: Option<PathBuf>,
    pub this_hash: Option<String>,
    pub binary_at: BinaryAt,
    pub link_at: LinkAt,
    pub first_mr: Option<(PathBuf, bool)>,
    pub first_mortar: Option<(PathBuf, bool)>,
    pub first_mr_at: Option<usize>,
    pub bin_at: Option<usize>,
    pub exe_dir: Option<PathBuf>,
    pub detected: Vec<&'static HarnessSpec>,
    pub skills: Vec<(&'static HarnessSpec, PathBuf, SkillAt)>,
    pub record: Vec<RecordItem>,
}
