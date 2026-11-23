//! with_base_status: base, partners and partner_moved after the report's own fields for every request; the base,
//! partners and moved lines only for a request with partners.

use folders::RequestPartner;
use gitio::Oid;

use super::{with_base_status, BaseStatus};
use crate::report::{ExitCode, OutValue, Report};

const A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const C: &str = "cccccccccccccccccccccccccccccccccccccccc";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).expect("a valid oid")
}

fn report() -> Report {
    Report {
        verb: "status".to_string(),
        exit: ExitCode::Done,
        outcome: "clean".to_string(),
        lines: vec!["request task in zone core".to_string()],
        owner: None,
        next: "mr join".to_string(),
        fields: vec![("gaps".to_string(), OutValue::Texts(vec![]))],
    }
}

fn status(partners: &[(&str, &str)], moved: &[&str]) -> BaseStatus {
    BaseStatus {
        base: oid(A),
        partners: partners.iter().map(|(zone, tip)| RequestPartner { zone: zone.to_string(), tip: oid(tip) }).collect(),
        moved: moved.iter().map(|zone| zone.to_string()).collect(),
    }
}

fn texts(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| item.to_string()).collect()
}

/// One row: its name, the status, the expected partners values, lines and partner_moved.
type Case = (&'static str, BaseStatus, Vec<String>, Vec<String>, bool);

#[test]
fn with_base_status_cases() {
    let web = format!("{{\"zone\":\"web\",\"tip\":\"{B}\"}}");
    let api = format!("{{\"zone\":\"api\",\"tip\":\"{C}\"}}");
    let cases: Vec<Case> = vec![
        ("plain request", status(&[], &[]), vec![], texts(&["request task in zone core"]), false),
        (
            "partners unmoved",
            status(&[("web", B)], &[]),
            vec![web.clone()],
            vec![
                "request task in zone core".to_string(),
                format!("base {A}"),
                format!("partners: web at {B}"),
            ],
            false,
        ),
        (
            "two partners, both moved",
            status(&[("web", B), ("api", C)], &["web", "api"]),
            vec![web, api],
            vec![
                "request task in zone core".to_string(),
                format!("base {A}"),
                format!("partners: web at {B}, api at {C}"),
                "partner zones moved since the base: web, api".to_string(),
            ],
            true,
        ),
    ];
    for (name, status, partners, lines, moved) in cases {
        let mut expected = report();
        expected.lines = lines;
        expected.fields.extend([
            ("base".to_string(), OutValue::Text(A.to_string())),
            ("partners".to_string(), OutValue::Raw(partners)),
            ("partner_moved".to_string(), OutValue::Bool(moved)),
        ]);
        assert_eq!(with_base_status(report(), &status), expected, "case {name}");
    }
}
