//! The report of an `mr would-merge <squash> --with <squash>...` that land's would_merge_with answered.

use std::path::PathBuf;

use gitio::Oid;
use land::{Squash, WouldMergeWith};

use crate::landing::CHANGE_CONFLICT_NEXT;
use crate::report::{ExitCode, OutValue, Report};

const VERB: &str = "would-merge";
const NOTHING_REACHED: &str = "no ref moved; any merge commit the trial wrote is reached by nothing";

/// Step 1 is `squash`, step k + 1 the k-th of `with`. Fields change_id and commit name `squash` in both answers.
/// - Clean: exit 0 "merges", a line for step 1 merging into integration and one for each later step merging onto
///   the step before; next lands each step's commit in order.
/// - Conflicted: exit 2 "conflict", naming the step that conflicts (and, past step 1, the step it came after) and
///   listing each conflicting file on its own line, owner the change's zone; next is CHANGE_CONFLICT_NEXT. Fields
///   also files and step (its number and commit).
///
/// Both end saying no ref moved and any merge commit the trial wrote is reached by nothing.
pub fn would_merge_with_report(squash: &Squash, with: &[Squash], answer: &WouldMergeWith) -> Report {
    let fields = vec![
        ("change_id".to_string(), OutValue::Text(squash.change_id.to_string())),
        ("commit".to_string(), OutValue::Text(squash.commit.to_string())),
    ];
    match answer {
        WouldMergeWith::Clean => clean_report(squash, with, fields),
        WouldMergeWith::Conflicted { step, commit, files } => {
            let step_squash = if *step == 1 { squash } else { &with[step - 2] };
            conflict_report(*step, step_squash, commit, files, fields)
        }
    }
}

fn clean_report(squash: &Squash, with: &[Squash], fields: Vec<(String, OutValue)>) -> Report {
    let mut lines = vec![format!("change {} (commit {}) merges cleanly into integration", squash.change_id, squash.commit)];
    lines.extend(with.iter().enumerate().map(|(index, further)| {
        format!("then change {} (commit {}) merges cleanly onto step {}", further.change_id, further.commit, index + 1)
    }));
    lines.push(NOTHING_REACHED.to_string());
    let lands: Vec<String> =
        std::iter::once(squash).chain(with).map(|step| format!("mr land {} --actor <name>", step.commit)).collect();
    Report {
        verb: VERB.to_string(),
        exit: ExitCode::Done,
        outcome: "merges".to_string(),
        lines,
        owner: None,
        next: lands.join(", then "),
        fields,
    }
}

/// `commit` is the answer's, the commit of `step_squash`, the squash at `step`.
fn conflict_report(
    step: usize,
    step_squash: &Squash,
    commit: &Oid,
    files: &[PathBuf],
    mut fields: Vec<(String, OutValue)>,
) -> Report {
    let id = &step_squash.change_id;
    let mut lines = vec![match step {
        1 => format!("step 1: change {id} (commit {commit}) conflicts with integration in:"),
        n => format!("step {n}: change {id} (commit {commit}) conflicts with integration after step {} in:", n - 1),
    }];
    let files: Vec<String> = files.iter().map(|file| file.display().to_string()).collect();
    lines.extend(files.iter().cloned());
    lines.push(NOTHING_REACHED.to_string());
    fields.push(("files".to_string(), OutValue::Texts(files)));
    fields.push((
        "step".to_string(),
        OutValue::Object(vec![
            ("number".to_string(), OutValue::Number(step as u64)),
            ("commit".to_string(), OutValue::Text(commit.to_string())),
        ]),
    ));
    Report {
        verb: VERB.to_string(),
        exit: ExitCode::Conflict,
        outcome: "conflict".to_string(),
        lines,
        owner: Some("the change's zone".to_string()),
        next: CHANGE_CONFLICT_NEXT.to_string(),
        fields,
    }
}
