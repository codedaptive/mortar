//! left_set_report: a resolve's report gains the left-the-set line, the left_partners field and next "mr join",
//! everything else kept.

use super::left_set_report;
use crate::report::{ExitCode, OutValue, Report};

fn given() -> Report {
    Report {
        verb: "resolve".to_string(),
        exit: ExitCode::Done,
        outcome: "rebased".to_string(),
        lines: vec!["rebased: the request moved".to_string()],
        owner: None,
        next: "run the unit tests".to_string(),
        fields: vec![("from".to_string(), OutValue::Text("1111111111111111111111111111111111111111".to_string()))],
    }
}

#[test]
fn left_set_report_cases() {
    let cases: Vec<(&str, Vec<String>, &str)> = vec![
        (
            "one partner",
            vec!["web".to_string()],
            "the request has left its set's merge: it stands on the zone tip without partners web; its unit tests \
             run without them, and the set's tests run it with them after mr join",
        ),
        (
            "two partners in order",
            vec!["web".to_string(), "api".to_string()],
            "the request has left its set's merge: it stands on the zone tip without partners web, api; its unit \
             tests run without them, and the set's tests run it with them after mr join",
        ),
    ];
    for (name, partners, line) in cases {
        let expected = Report {
            verb: "resolve".to_string(),
            exit: ExitCode::Done,
            outcome: "rebased".to_string(),
            lines: vec!["rebased: the request moved".to_string(), line.to_string()],
            owner: None,
            next: "mr join".to_string(),
            fields: vec![
                ("from".to_string(), OutValue::Text("1111111111111111111111111111111111111111".to_string())),
                ("left_partners".to_string(), OutValue::Texts(partners.clone())),
            ],
        };
        assert_eq!(left_set_report(given(), &partners), expected, "case {name}");
    }
}
