//! overlap_report: the report of a `mortar overlap <zone>` that read the zone's open requests (exit 0 either way).

use ::join::OverlapPair;

use crate::report::{ExitCode, OutValue, Report};

/// Done, next "nothing to do". No pair: "no-overlap", one line saying so for `zone`. Else "overlaps", one line per
/// pair in the order given, naming both requests and the paths both change. Field pairs: "<first> <second>" per pair.
pub fn overlap_report(zone: &str, pairs: &[OverlapPair]) -> Report {
    let (word, lines) = if pairs.is_empty() {
        ("no-overlap", vec![format!("no two open requests of zone {zone} change the same path")])
    } else {
        (
            "overlaps",
            pairs
                .iter()
                .map(|pair| {
                    let paths = pair.paths.iter().map(|path| path.display().to_string()).collect::<Vec<_>>();
                    format!("requests {} and {} both change: {}", pair.first, pair.second, paths.join(", "))
                })
                .collect(),
        )
    };
    Report {
        verb: "overlap".to_string(),
        exit: ExitCode::Done,
        outcome: word.to_string(),
        lines,
        owner: None,
        next: "nothing to do".to_string(),
        fields: vec![(
            "pairs".to_string(),
            OutValue::Texts(pairs.iter().map(|pair| format!("{} {}", pair.first, pair.second)).collect()),
        )],
    }
}
