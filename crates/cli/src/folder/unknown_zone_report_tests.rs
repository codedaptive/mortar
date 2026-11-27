//! unknown_zone_report: a zone the config does not name is refused exit 3, its next init's --replace command with
//! the zone added.

use std::collections::BTreeMap;

use super::unknown_zone_report;
use crate::config::{Config, ZoneConfig};
use crate::report::ExitCode;

#[test]
fn a_zone_not_in_the_config_is_refused_with_init_replace_next() {
    let config = Config {
        origin_branch: "main".to_string(),
        integration_branch: "integration".to_string(),
        remote: "origin".to_string(),
        zones: BTreeMap::from([("core".to_string(), ZoneConfig { paths: vec!["src".to_string()] })]),
        root_files: Vec::new(),
        build_dirs: Vec::new(),
        cap_bytes: 1,
        floor_bytes: 0,
        formatter: None,
    };
    let report = unknown_zone_report("docs", &config);
    assert_eq!((report.exit, report.outcome.as_str()), (ExitCode::Refused, "refused"));
    assert!(report.next.starts_with("mr init --replace"), "{}", report.next);
    assert!(report.next.contains("--zone docs="), "{}", report.next);
}
