//! zone_paths_record: a zone record gets the new paths and no needed paths; any other record is refused.

use std::path::{Path, PathBuf};

use gitio::Oid;

use super::zone_paths_record::zone_paths_record;
use super::FolderError;
use crate::record::{FolderKind, FolderRecord};

fn strings(given: &[&str]) -> Vec<String> {
    given.iter().map(|s| s.to_string()).collect()
}

fn base() -> Oid {
    Oid::parse("489d82745705a031306253da29bdad7aee19d2e5").unwrap()
}

fn zone_record() -> FolderRecord {
    FolderRecord {
        kind: FolderKind::Zone,
        zone: "red".to_string(),
        request: None,
        base: base(),
        work_order_paths: strings(&["kit/zone"]),
        needed_paths: strings(&["needed"]),
        parent: None,
    }
}

fn request_record() -> FolderRecord {
    FolderRecord {
        kind: FolderKind::Request,
        zone: "red".to_string(),
        request: Some("one".to_string()),
        base: base(),
        work_order_paths: strings(&["kit/zone"]),
        needed_paths: Vec::new(),
        parent: Some("zone/red".to_string()),
    }
}

/// The record zone_paths_record answers, or the path and record its DifferentFolder carries.
type Expected = Result<FolderRecord, (PathBuf, FolderRecord)>;

#[test]
fn a_zone_record_takes_the_new_paths_and_any_other_is_refused() {
    let folder = Path::new("/work/folder");
    let rows: [(&str, FolderRecord, Expected); 2] = [
        (
            "zone record with needed paths",
            zone_record(),
            Ok(FolderRecord {
                kind: FolderKind::Zone,
                zone: "red".to_string(),
                request: None,
                base: base(),
                work_order_paths: strings(&["kit/dependency", "kit/zone"]),
                needed_paths: Vec::new(),
                parent: None,
            }),
        ),
        (
            "request record",
            request_record(),
            Err((PathBuf::from("/work/folder"), request_record())),
        ),
    ];
    for (case, existing, expected) in rows {
        let actual = zone_paths_record(folder, &existing, &strings(&["kit/dependency", "kit/zone"]));
        match (actual, expected) {
            (Ok(record), Ok(want)) => assert_eq!(record, want, "{case}"),
            (Err(FolderError::DifferentFolder { path, existing }), Err((want_path, want_existing))) => {
                assert_eq!(path, want_path, "{case}");
                assert_eq!(existing, want_existing, "{case}");
            }
            (other, _) => panic!("{case}: unexpected {other:?}"),
        }
    }
}
