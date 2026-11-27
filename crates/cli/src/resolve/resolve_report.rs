//! The report of an `mr resolve` that ran to an answer.
//! resolve_report_tests covers it (it calls join's answer_part and finished_move_part and report::failure_report,
//! which are pure, directly).

use std::path::PathBuf;

use ::join::JoinAnswer;
use gitio::Oid;

use super::ResolveOutcome;
use crate::join::{answer_part, finished_move_part, moved_before, AnswerPart};
use crate::report::{failure_report, ExitCode, OutValue, Report};

const VERB: &str = "resolve";

/// Lines: finished_move_part's, then the answer's. Fields: finished_move_part's, request_ref, then the answer's.
/// FilesWritten (the first run) is exit 1 "files-written": the commit whose conflict was written, each file written,
/// each path skipped for having no version in the conflicted tree and each submodule path with its own step;
/// next "fix the files, then mr resolve". Rebased and Conflicted report as join does (answer_part), and
/// Rebased adds the snapshot's ref and commit. FilesWritten and Conflicted name the snapshot ref when the snapshot
/// before resolve moved it (moved_before; resolve saves nothing, so the request branch never moved before it).
/// Joined and FastForwarded, which resolve never gives, are failure_report.
pub fn resolve_report(outcome: &ResolveOutcome) -> Report {
    let moved = moved_before(false, outcome.snapshot_written, &outcome.request_ref);
    let mut part = match (&outcome.answer, answer_part(&outcome.answer, &outcome.request_ref, moved.as_deref())) {
        (JoinAnswer::FilesWritten { commit, files, skipped, gitlinks }, _) => {
            files_written(commit, files, skipped, gitlinks, moved.as_deref())
        }
        (JoinAnswer::Rebased { .. } | JoinAnswer::Conflicted { .. }, Some(part)) => part,
        _ => return failure_report(VERB, "join resolve answered joined, which it never gives"),
    };
    if matches!(outcome.answer, JoinAnswer::Rebased { .. }) {
        part.fields.push(("snapshot_ref".to_string(), OutValue::Text(outcome.snapshot_ref.clone())));
        part.fields.push(("snapshot_commit".to_string(), OutValue::Text(outcome.snapshot_commit.to_string())));
    }
    let (mut lines, mut fields) = finished_move_part(outcome.finished_move.as_ref());
    lines.extend(part.lines);
    fields.push(("request_ref".to_string(), OutValue::Text(outcome.request_ref.clone())));
    fields.extend(part.fields);
    Report {
        verb: VERB.to_string(),
        exit: part.exit,
        outcome: part.outcome.to_string(),
        lines,
        owner: None,
        next: part.next.to_string(),
        fields,
    }
}

/// The first run's part. Lines: the commit with what moved (`moved` before "no other ref moved", or "no ref moved"),
/// each file written (indented), each skipped path, each submodule path with its step. Fields: commit, files,
/// skipped, gitlinks, each path list in join's order.
fn files_written(
    commit: &Oid,
    files: &[PathBuf],
    skipped: &[PathBuf],
    gitlinks: &[PathBuf],
    moved: Option<&str>,
) -> AnswerPart {
    let texts = |paths: &[PathBuf]| paths.iter().map(|path| path.display().to_string()).collect::<Vec<_>>();
    let refs = moved.map_or("no ref moved".to_string(), |moved| format!("{moved}; no other ref moved"));
    let mut lines =
        vec![format!("files written: the conflicted versions of commit {commit}, markers included; {refs}")];
    lines.extend(texts(files).into_iter().map(|path| format!("  {path}")));
    lines.extend(texts(skipped).into_iter().map(|path| format!("skipped (no version in the conflicted tree): {path}")));
    lines.extend(
        texts(gitlinks)
            .into_iter()
            .map(|path| format!("submodule {path}: set its commit in the folder, then mr resolve")),
    );
    AnswerPart {
        lines,
        fields: vec![
            ("commit".to_string(), OutValue::Text(commit.to_string())),
            ("files".to_string(), OutValue::Texts(texts(files))),
            ("skipped".to_string(), OutValue::Texts(texts(skipped))),
            ("gitlinks".to_string(), OutValue::Texts(texts(gitlinks))),
        ],
        exit: ExitCode::Conflict,
        outcome: "files-written",
        next: "fix the files, then mr resolve",
    }
}
