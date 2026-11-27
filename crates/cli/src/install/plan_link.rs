//! plan_link: Install's decision about the short-name link `~/.local/bin/mr`.
//! Pure: it decides from the facts install read. In order: the link's target (none when the binary is not placed
//! and no `mortar` on PATH is this binary: the link is skipped and nothing else is asked); then the clash (a foreign
//! first `mr` on PATH without --short-name: skipped, never a refusal); only then what stands at the link's path,
//! where a foreign file is the one refusal.

use std::path::Path;

use crate::report::{ExitCode, Report};

use super::{InstallFacts, InstallItem, InstallPaths, InstallStep, LinkAt, LinkPlan};

/// The link's plan. `placed`: the binary is placed, so the target is `paths.mortar`; else the first `mortar` on PATH
/// when it is Mortar, as PATH names it. `short_name`: link even behind a foreign first `mr`. Err the refusal (exit 3)
/// when install would make the link over a file that is neither Mortar nor a link to it.
pub fn plan_link(facts: &InstallFacts, paths: &InstallPaths, placed: bool, short_name: bool) -> Result<LinkPlan, Report> {
    let skipped = |note: String, present: bool, next: Option<String>| LinkPlan {
        step: None,
        item: InstallItem { action: "link skipped".to_string(), path: paths.mr.clone(), note },
        present,
        next,
    };
    let target = match (placed, &facts.first_mortar) {
        (true, _) => paths.mortar.clone(),
        (false, Some((first, true))) => first.clone(),
        (false, _) => {
            let next = facts.exe_dir.as_ref().map_or_else(
                || "add the package's bin folder to PATH".to_string(),
                |dir| format!("add {} to PATH", dir.display()),
            );
            return Ok(skipped("no mortar on PATH is this binary".to_string(), false, Some(next)));
        }
    };
    if let (Some((first, false)), false) = (&facts.first_mr, short_name) {
        let note = format!(
            "the first mr on PATH is {}, another program: run Mortar as mortar, or link anyway with --short-name",
            first.display()
        );
        let present = matches!(&facts.link_at, LinkAt::Link { target: at, .. } if *at == target);
        return Ok(skipped(note, present, None));
    }
    let (action, step) = match &facts.link_at {
        LinkAt::Link { target: at, .. } if *at == target => ("unchanged", None),
        LinkAt::Absent | LinkAt::Link { mortar: true, .. } | LinkAt::Mortar => {
            ("linked", Some(InstallStep::Link { at: paths.mr.clone(), target: target.clone() }))
        }
        LinkAt::Link { mortar: false, .. } | LinkAt::Foreign => return Err(foreign(&paths.mr)),
    };
    let note = format!("-> {}", target.display());
    let item = InstallItem { action: action.to_string(), path: paths.mr.clone(), note };
    Ok(LinkPlan { step, item, present: true, next: None })
}

/// The refusal of a file at `~/.local/bin/mr` that is neither Mortar nor a link to it.
fn foreign(mr: &Path) -> Report {
    let mr = mr.display();
    Report {
        verb: "install".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![format!("{mr} is another program, not Mortar's link"), "nothing changed".to_string()],
        owner: Some(format!("whoever put {mr} there")),
        next: format!("move {mr} aside, then install again"),
        fields: Vec::new(),
    }
}
