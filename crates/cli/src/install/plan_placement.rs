//! plan_placement: Install's decision about placing the binary at `~/.local/bin/mortar`.
//! Pure: it decides from the facts install read. A skip reason decides first; only without one does what stands
//! at the path count, and a foreign program there is the one refusal.

use crate::report::{ExitCode, Report};

use super::{BinaryAt, InstallFacts, InstallItem, InstallPaths, InstallStep};

/// `skip` Some: no step, the line "skipped <mortar>: <reason>". Else by what stands at `paths.mortar`: nothing, a
/// step and "placed"; another Mortar build, a step and "replaced"; this binary, no step and "unchanged"; a foreign
/// program, Err the refusal (exit 3). The step copies `facts.this` to `paths.mortar`.
pub fn plan_placement(
    facts: &InstallFacts,
    paths: &InstallPaths,
    skip: Option<&str>,
) -> Result<(Option<InstallStep>, InstallItem), Report> {
    let item = |action: &str, note: &str| InstallItem {
        action: action.to_string(),
        path: paths.mortar.clone(),
        note: note.to_string(),
    };
    let step = || {
        facts.this.clone().map(|from| InstallStep::PlaceBinary { from, to: paths.mortar.clone() })
    };
    if let Some(reason) = skip {
        return Ok((None, item("skipped", reason)));
    }
    match facts.binary_at {
        BinaryAt::Absent => Ok((step(), item("placed", ""))),
        BinaryAt::Mortar => Ok((step(), item("replaced", ""))),
        BinaryAt::Same => Ok((None, item("unchanged", ""))),
        BinaryAt::Foreign => Err(foreign(paths)),
    }
}

/// The refusal of a program at `~/.local/bin/mortar` that is not Mortar.
fn foreign(paths: &InstallPaths) -> Report {
    let mortar = paths.mortar.display();
    Report {
        verb: "install".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![format!("{mortar} is another program, not Mortar"), "nothing changed".to_string()],
        owner: Some(format!("whoever put {mortar} there")),
        next: format!("move {mortar} aside, or install with --no-place"),
        fields: Vec::new(),
    }
}
