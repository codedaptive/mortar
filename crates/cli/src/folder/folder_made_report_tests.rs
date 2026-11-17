//! folder_made_report: a zone and a request folder made, each with its lines, fields and next.

use std::path::Path;

use folders::{FolderKind, FolderRecord};
use gitio::Oid;

use super::folder_made_report;
use crate::report::{ExitCode, OutValue, Report};

const BASE: &str = "abababababababababababababababababababab";

fn record(kind: FolderKind, request: Option<&str>, needed: &[&str]) -> FolderRecord {
    FolderRecord {
        kind,
        zone: "core".to_string(),
        request: request.map(str::to_string),
        base: Oid::parse(BASE).expect("a valid oid"),
        work_order_paths: vec!["Cargo.toml".to_string(), "src/a".to_string()],
        needed_paths: needed.iter().map(ToString::to_string).collect(),
        parent: request.map(|_| "zone/core".to_string()),
    }
}

fn text(value: &str) -> OutValue {
    OutValue::Text(value.to_string())
}

#[test]
fn a_zone_folder_made_names_its_zone_base_and_cone_and_next_makes_request_folders() {
    assert_eq!(
        folder_made_report("zone", Path::new("/w/core"), &record(FolderKind::Zone, None, &[])),
        Report {
            verb: "folder".to_string(),
            exit: ExitCode::Done,
            outcome: "made".to_string(),
            lines: vec![
                "the zone folder of zone core is at /w/core".to_string(),
                format!("base {BASE}"),
                "cone Cargo.toml, src/a".to_string(),
            ],
            owner: None,
            next: "make request folders with mortar folder request core <request> <path> --paths <path>... --actor \
                   <name>"
                .to_string(),
            fields: vec![
                ("path".to_string(), text("/w/core")),
                ("kind".to_string(), text("zone")),
                ("zone".to_string(), text("core")),
                ("request".to_string(), OutValue::Null),
                ("base".to_string(), text(BASE)),
                ("paths".to_string(), OutValue::Texts(vec!["Cargo.toml".to_string(), "src/a".to_string()])),
            ],
        }
    );
}

#[test]
fn a_request_folder_made_names_its_request_and_next_starts_its_agent_there() {
    let record = record(FolderKind::Request, Some("fix-1"), &["docs"]);
    let report = folder_made_report("request", Path::new("/w/fix"), &record);
    assert_eq!(report.exit, ExitCode::Done);
    assert_eq!(report.outcome, "made");
    assert_eq!(
        report.lines,
        vec![
            "the request folder of request fix-1 in zone core is at /w/fix".to_string(),
            format!("base {BASE}"),
            "cone Cargo.toml, src/a, docs".to_string(),
        ]
    );
    assert_eq!(report.next, "start the request's agent in /w/fix");
    assert_eq!(report.fields[1], ("kind".to_string(), text("request")));
    assert_eq!(report.fields[3], ("request".to_string(), text("fix-1")));
    assert_eq!(
        report.fields[5],
        ("paths".to_string(), OutValue::Texts(vec!["Cargo.toml".to_string(), "src/a".to_string(), "docs".to_string()]))
    );
}
