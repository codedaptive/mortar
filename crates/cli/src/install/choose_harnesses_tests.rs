//! choose_harnesses: --target, then no harness detected, then -y, then no terminal, then the picker; `ask` only on
//! the picker (spec 07 item 4).

use super::{choose_harnesses, HarnessSpec, InstallOptions, SkillLocation, HARNESSES};
use crate::report::{ExitCode, Report};

type Chosen = Result<(Vec<&'static HarnessSpec>, Option<String>), Report>;

const NO_HARNESS: &str = "no harness folder (.claude, .codex, .cursor) in HOME";
const NO_TERMINAL: &str =
    "no terminal for the picker: name harnesses with --target <list>, or take every detected one with -y";

fn options(targets: Option<Vec<&'static HarnessSpec>>, yes: bool) -> InstallOptions {
    InstallOptions { no_place: false, short_name: false, targets, yes, location: SkillLocation::Global }
}

/// Runs choose_harnesses with a fake `ask` answering `answer`; also returns every prompt it was asked.
fn choose(detected: &[&'static HarnessSpec], options: &InstallOptions, terminal: bool, answer: Option<&str>) -> (Chosen, Vec<String>) {
    let mut prompts = Vec::new();
    let mut ask = |prompt: &str| {
        prompts.push(prompt.to_string());
        answer.map(str::to_string)
    };
    let chosen = choose_harnesses(detected, options, terminal, &mut ask);
    (chosen, prompts)
}

fn names(chosen: &Chosen) -> Vec<&'static str> {
    chosen.as_ref().expect("chosen").0.iter().map(|h| h.name).collect()
}

fn note(chosen: &Chosen) -> Option<&str> {
    chosen.as_ref().expect("chosen").1.as_deref()
}

#[test]
fn detected_targets_are_wired_without_asking_even_on_a_terminal() {
    let (chosen, prompts) = choose(&[&HARNESSES[0], &HARNESSES[1]], &options(Some(vec![&HARNESSES[1]]), false), true, Some("all"));
    assert_eq!(names(&chosen), vec!["codex"]);
    assert_eq!(note(&chosen), None);
    assert!(prompts.is_empty());
}

#[test]
fn a_target_not_detected_is_refused_before_the_empty_detection_and_y() {
    let (chosen, prompts) = choose(&[], &options(Some(vec![&HARNESSES[2]]), true), true, Some("all"));
    let report = chosen.expect_err("refused");
    assert_eq!(report.verb, "install");
    assert_eq!(report.exit, ExitCode::Refused);
    assert_eq!(report.outcome, "refused");
    assert_eq!(report.lines, vec!["cursor is not detected: there is no .cursor in HOME", "nothing changed"]);
    assert_eq!(report.owner.as_deref(), Some("the person installing Mortar"));
    assert_eq!(report.next, "leave cursor out of --target, or install cursor first");
    assert!(prompts.is_empty());
}

#[test]
fn nothing_detected_is_noted_before_y_and_the_terminal() {
    for (yes, terminal) in [(true, true), (false, true), (false, false)] {
        let (chosen, prompts) = choose(&[], &options(None, yes), terminal, Some("all"));
        assert!(names(&chosen).is_empty());
        assert_eq!(note(&chosen), Some(NO_HARNESS));
        assert!(prompts.is_empty());
    }
}

#[test]
fn y_wires_every_detected_one_without_asking_with_or_without_a_terminal() {
    for terminal in [true, false] {
        let (chosen, prompts) = choose(&[&HARNESSES[0], &HARNESSES[2]], &options(None, true), terminal, Some(""));
        assert_eq!(names(&chosen), vec!["claude", "cursor"]);
        assert_eq!(note(&chosen), None);
        assert!(prompts.is_empty());
    }
}

#[test]
fn no_terminal_wires_nothing_and_says_how() {
    let (chosen, prompts) = choose(&[&HARNESSES[0]], &options(None, false), false, Some("all"));
    assert!(names(&chosen).is_empty());
    assert_eq!(note(&chosen), Some(NO_TERMINAL));
    assert!(prompts.is_empty());
}

#[test]
fn the_picker_asks_once_with_the_detected_names() {
    let (chosen, prompts) = choose(&[&HARNESSES[0], &HARNESSES[1]], &options(None, false), true, Some("codex\n"));
    assert_eq!(names(&chosen), vec!["codex"]);
    assert_eq!(note(&chosen), None);
    assert_eq!(
        prompts,
        vec!["wire mortar's skill into which harnesses? detected: claude, codex; a comma list, all, or nothing for none: "]
    );
}

#[test]
fn the_end_of_input_or_an_empty_answer_picks_none() {
    for answer in [None, Some(""), Some("\n")] {
        let (chosen, prompts) = choose(&[&HARNESSES[0]], &options(None, false), true, answer);
        assert!(names(&chosen).is_empty());
        assert_eq!(note(&chosen), Some("none picked"));
        assert_eq!(prompts.len(), 1);
    }
}

#[test]
fn a_bad_answer_is_a_usage_report_for_install() {
    let (chosen, prompts) = choose(&[&HARNESSES[0]], &options(None, false), true, Some("cursor"));
    let report = chosen.expect_err("usage");
    assert_eq!(report.verb, "install");
    assert_eq!(report.exit, ExitCode::Usage);
    assert_eq!(report.outcome, "usage");
    assert_eq!(report.lines[0], "\"cursor\" is not a detected harness (detected: claude)");
    assert_eq!(prompts.len(), 1);
}
