//! plan_skills: Install's decision about each chosen harness's skill file (spec 07 items 4 and 5).
//! Pure: it decides from the facts install read. The chosen harnesses are taken in the order of `facts.skills`; the
//! first file that is not Mortar's skill refuses the whole plan, and the wiring-skipped line comes last.

use std::path::Path;

use crate::report::{ExitCode, Report};

use super::{content_hash, HarnessSpec, InstallFacts, InstallItem, InstallStep, RecordItem, SkillAt};

/// For each chosen harness, by what stands at its skill path: nothing, a write and "written"; a file holding `text`,
/// "unchanged"; another file the record lists, a write and "replaced"; anything else, Err the refusal (exit 1).
/// Each line names the harness. `wiring_note` Some: then the line "wiring skipped: <note>".
pub fn plan_skills(
    facts: &InstallFacts,
    chosen: &[&'static HarnessSpec],
    text: &str,
    wiring_note: Option<String>,
) -> Result<(Vec<InstallStep>, Vec<InstallItem>), Report> {
    let hash = content_hash(text.as_bytes());
    let mut steps = Vec::new();
    let mut items = Vec::new();
    for (harness, path, at) in facts.skills.iter().filter(|(harness, ..)| chosen.contains(harness)) {
        let recorded = facts.record.iter().any(|item| matches!(item, RecordItem::File { path: p, .. } if p == path));
        let action = match at {
            SkillAt::Absent => "written",
            SkillAt::File { hash: found } if *found == hash => "unchanged",
            SkillAt::File { .. } if recorded => "replaced",
            SkillAt::File { .. } | SkillAt::NotFile => return Err(foreign(path, harness)),
        };
        if action != "unchanged" {
            steps.push(InstallStep::WriteSkill { path: path.clone(), text: text.to_string() });
        }
        items.push(InstallItem { action: action.to_string(), path: path.clone(), note: harness.name.to_string() });
    }
    if let Some(note) = wiring_note {
        items.push(InstallItem { action: "wiring skipped".to_string(), path: Default::default(), note });
    }
    Ok((steps, items))
}

/// The refusal of a file at a skill path that is not Mortar's skill.
fn foreign(path: &Path, harness: &HarnessSpec) -> Report {
    let path = path.display();
    Report {
        verb: "install".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![format!("{path} is not Mortar's skill file"), "nothing changed".to_string()],
        owner: Some(format!("whoever wrote {path}")),
        next: format!("move {path} aside, or leave {} out of --target", harness.name),
        fields: Vec::new(),
    }
}
