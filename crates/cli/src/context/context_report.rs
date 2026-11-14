//! The report of a verb whose Mortar folder cannot be read.

use folders::FolderError;

use super::ContextError;
use crate::report::{failure_report, ExitCode, Report};

const OWNER: &str = "the harness that makes Mortar folders";

/// NotARepository and NotAFolder: refused (exit 1), outcome "not-a-folder", next "run mortar <verb> inside a
/// folder Mortar made". A record Mortar did not write: refused, naming the file and why. A record that could not
/// be read, any other folder error, and git failing: failure (exit 5).
pub fn context_report(verb: &str, error: &ContextError) -> Report {
    match error {
        ContextError::NotARepository { dir, error } => not_a_folder(
            verb,
            vec![
                format!("this directory is not in a Mortar folder: {} is in no git repository", dir.display()),
                error.to_string(),
            ],
        ),
        ContextError::NotAFolder { toplevel } => not_a_folder(
            verb,
            vec![format!(
                "this directory is not in a Mortar folder: the worktree {} has no Mortar record",
                toplevel.display()
            )],
        ),
        ContextError::Record(FolderError::Record { path, error }) => Report {
            verb: verb.to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec![
                format!("the folder record {} is not one Mortar writes: {}", path.display(), error.reason),
                "nothing ran".to_string(),
            ],
            owner: Some(OWNER.to_string()),
            next: "ask the harness that made this folder to make it again".to_string(),
            fields: Vec::new(),
        },
        ContextError::Record(other) => failure_report(verb, &other.to_string()),
        ContextError::Git(git) => failure_report(verb, &git.to_string()),
    }
}

fn not_a_folder(verb: &str, lines: Vec<String>) -> Report {
    Report {
        verb: verb.to_string(),
        exit: ExitCode::Refused,
        outcome: "not-a-folder".to_string(),
        lines,
        owner: Some(OWNER.to_string()),
        next: format!("run mortar {verb} inside a folder Mortar made"),
        fields: Vec::new(),
    }
}
