//! The report of a verb whose Mortar folder cannot be read.

use folders::FolderError;

use super::ContextError;
use crate::report::{failure_report, ExitCode, Report};

const OWNER: &str = "the harness that makes Mortar folders";

/// NotARepository: usage (exit 2), outcome "not-a-repository", naming the directory and git's error, next
/// "run mr <verb> inside the git repository". NotAFolder: refused (exit 3), outcome "not-a-folder", next "run mr
/// <verb> inside a folder Mortar made". A record Mortar did not write: refused, naming the file and why. A record
/// that could not be read, any other folder error, and git failing: failure (exit 5).
pub fn context_report(verb: &str, error: &ContextError) -> Report {
    match error {
        ContextError::NotARepository { dir, error } => Report {
            verb: verb.to_string(),
            exit: ExitCode::Usage,
            outcome: "not-a-repository".to_string(),
            lines: vec![format!("{} is in no git repository", dir.display()), error.to_string()],
            owner: Some(OWNER.to_string()),
            next: format!("run mr {verb} inside the git repository"),
            fields: Vec::new(),
        },
        ContextError::NotAFolder { toplevel } => Report {
            verb: verb.to_string(),
            exit: ExitCode::Refused,
            outcome: "not-a-folder".to_string(),
            lines: vec![format!(
                "this directory is not in a Mortar folder: the worktree {} has no Mortar record",
                toplevel.display()
            )],
            owner: Some(OWNER.to_string()),
            next: format!("run mr {verb} inside a folder Mortar made"),
            fields: Vec::new(),
        },
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
