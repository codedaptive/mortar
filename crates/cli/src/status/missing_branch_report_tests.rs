//! missing_branch_report: a missing zone branch is the zone coordinator's, a missing request branch this request's;
//! both exit 1 "refused", naming the branch and the verb to run again.

use super::missing_branch_report;
use crate::report::{ExitCode, Report};

#[test]
fn a_missing_zone_branch_is_the_zone_coordinators() {
    assert_eq!(
        missing_branch_report("status", "refs/heads/zone/core", true),
        Report {
            verb: "status".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec!["the branch refs/heads/zone/core does not exist".to_string()],
            owner: Some("the zone's coordinator".to_string()),
            next: "have the zone made, then mr status".to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn a_missing_request_branch_is_this_requests() {
    assert_eq!(
        missing_branch_report("need", "refs/heads/req/core/fix-1", false),
        Report {
            verb: "need".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec!["the branch refs/heads/req/core/fix-1 does not exist".to_string()],
            owner: Some("this request".to_string()),
            next: "make the request folder again, then mr need".to_string(),
            fields: Vec::new(),
        }
    );
}
