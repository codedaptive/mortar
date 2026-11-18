//! choose_harnesses: Which harnesses install wires: --target, -y, the picker, or none (spec 07 item 4).

use crate::args::{usage_report, UsageError};
use crate::report::{ExitCode, Report};

use super::{parse_picker_answer, HarnessSpec, InstallOptions};

/// The harnesses to wire and, when none are wired for a reason, the wiring-skipped note; Err a refusal (a target not
/// detected, exit 1) or a usage report (a bad picker answer, exit 4). Checked in order: --target, no harness
/// detected, -y, no terminal, then the picker. `ask` is called at most once, only for the picker on a terminal.
pub fn choose_harnesses(
    detected: &[&'static HarnessSpec],
    options: &InstallOptions,
    terminal: bool,
    ask: &mut dyn FnMut(&str) -> Option<String>,
) -> Result<(Vec<&'static HarnessSpec>, Option<String>), Report> {
    if let Some(targets) = &options.targets {
        return match targets.iter().find(|target| !detected.contains(target)) {
            Some(missing) => Err(not_detected(missing)),
            None => Ok((targets.clone(), None)),
        };
    }
    if detected.is_empty() {
        return Ok((Vec::new(), Some("no harness folder (.claude, .codex, .cursor) in HOME".to_string())));
    }
    if options.yes {
        return Ok((detected.to_vec(), None));
    }
    if !terminal {
        let note = "no terminal for the picker: name harnesses with --target <list>, or take every detected one with -y";
        return Ok((Vec::new(), Some(note.to_string())));
    }
    let names = detected.iter().map(|harness| harness.name).collect::<Vec<_>>().join(", ");
    let prompt =
        format!("wire mortar's skill into which harnesses? detected: {names}; a comma list, all, or nothing for none: ");
    let none_picked = || Ok((Vec::new(), Some("none picked".to_string())));
    let Some(answer) = ask(&prompt) else {
        return none_picked();
    };
    match parse_picker_answer(&answer, detected) {
        Ok(picked) if picked.is_empty() => none_picked(),
        Ok(picked) => Ok((picked, None)),
        Err(reason) => Err(usage_report(&UsageError { verb: Some("install".to_string()), reason, json: false })),
    }
}

/// The refusal of a --target harness whose folder is not in HOME.
fn not_detected(harness: &HarnessSpec) -> Report {
    Report {
        verb: "install".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![
            format!("{} is not detected: there is no {} in HOME", harness.name, harness.folder),
            "nothing changed".to_string(),
        ],
        owner: Some("the person installing Mortar".to_string()),
        next: format!("leave {0} out of --target, or install {0} first", harness.name),
        fields: Vec::new(),
    }
}
