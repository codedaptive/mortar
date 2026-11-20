//! skill_path: spec 07 item 5's table, global under HOME and local under the project.

use std::path::{Path, PathBuf};

use super::{skill_path, SkillLocation, HARNESSES};

const HOME: &str = "/home/ana";
const PROJECT: &str = "/work/repo";

fn path_for(index: usize, location: SkillLocation) -> PathBuf {
    skill_path(&HARNESSES[index], location, Path::new(HOME), Path::new(PROJECT))
}

#[test]
fn global_paths_are_under_home() {
    assert_eq!(path_for(0, SkillLocation::Global), PathBuf::from("/home/ana/.claude/skills/mortar/SKILL.md"));
    assert_eq!(path_for(1, SkillLocation::Global), PathBuf::from("/home/ana/.agents/skills/mortar/SKILL.md"));
    assert_eq!(path_for(2, SkillLocation::Global), PathBuf::from("/home/ana/.cursor/skills/mortar/SKILL.md"));
}

#[test]
fn local_paths_are_under_the_project() {
    assert_eq!(path_for(0, SkillLocation::Local), PathBuf::from("/work/repo/.claude/skills/mortar/SKILL.md"));
    assert_eq!(path_for(1, SkillLocation::Local), PathBuf::from("/work/repo/.agents/skills/mortar/SKILL.md"));
    assert_eq!(path_for(2, SkillLocation::Local), PathBuf::from("/work/repo/.cursor/skills/mortar/SKILL.md"));
}
