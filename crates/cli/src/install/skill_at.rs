//! What stands where install would write a harness's skill file.
//! A bare type; plan_skills_tests exercises it.

/// Absent: nothing there. File: a regular file, with content_hash of its bytes. NotFile: anything else.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillAt {
    Absent,
    File { hash: String },
    NotFile,
}
