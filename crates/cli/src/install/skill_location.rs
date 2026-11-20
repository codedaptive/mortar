//! Where install writes the skill file: `--location global` or `--location local`.
//! A bare type; skill_path_tests and parse_install_args_tests exercise it.

/// Global: under HOME, the default. Local: under the project (the git toplevel holding the current directory, or the
/// current directory outside a repository).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillLocation {
    Global,
    Local,
}
