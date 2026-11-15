//! missing_path_report: `mortar need`'s refusal of a path the repository does not hold (spec 02, S-90(2)).

use gitio::Oid;

use crate::report::{ExitCode, OutValue, Report};

/// Exit 1 "refused": `path` is not in the repository at the folder's commit `commit`, and nothing ran (the check
/// runs before the lock). Owned by this request; next names a path the repository holds. Fields path and commit.
pub fn missing_path_report(path: &str, commit: &Oid) -> Report {
    Report {
        verb: "need".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![
            format!("{path} is not in the repository at the folder's commit {}", commit.as_str()),
            "nothing ran".to_string(),
        ],
        owner: Some("this request".to_string()),
        next: "mortar need <a path the repository holds>".to_string(),
        fields: vec![
            ("path".to_string(), OutValue::Text(path.to_string())),
            ("commit".to_string(), OutValue::Text(commit.as_str().to_string())),
        ],
    }
}
