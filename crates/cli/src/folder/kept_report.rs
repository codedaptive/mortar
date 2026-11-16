//! The report of a `mortar folder keep` that kept an attempt, or finished the request folder's move a killed keep
//! left undone.

use std::path::PathBuf;

use gitio::Oid;

use crate::report::{ExitCode, OutValue, Report};

/// Exit 0; outcome "kept", or "already-done" when `finished` (an earlier keep of attempt `k` had moved the branch,
/// and this run finished only the request folder's move). Lines: each re-made orphan attempt folder in `remade`;
/// the request branch's move from `old` to `new` (full commit ids); that the request folder now holds attempt k's
/// work. Fields: k, old, new, remade. Next: go on with the request in its own folder.
pub fn kept_report(k: u32, old: &Oid, new: &Oid, remade: &[PathBuf], finished: bool) -> Report {
    let (old, new) = (old.as_str(), new.as_str());
    let mut lines: Vec<String> =
        remade.iter().map(|path| format!("re-made the orphan attempt folder {}", path.display())).collect();
    lines.push(if finished {
        format!("attempt {k} was kept before: the request branch had moved from {old} to {new}")
    } else {
        format!("kept attempt {k}: the request branch moved from {old} to {new}")
    });
    lines.push(format!("the request folder holds attempt {k}'s work; the attempt folders and try branches are gone"));
    Report {
        verb: "folder".to_string(),
        exit: ExitCode::Done,
        outcome: if finished { "already-done" } else { "kept" }.to_string(),
        lines,
        owner: None,
        next: "go on with the request in its own folder".to_string(),
        fields: vec![
            ("k".to_string(), OutValue::Number(u64::from(k))),
            ("old".to_string(), OutValue::Text(old.to_string())),
            ("new".to_string(), OutValue::Text(new.to_string())),
            ("remade".to_string(), OutValue::Texts(remade.iter().map(|path| path.display().to_string()).collect())),
        ],
    }
}
