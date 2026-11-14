//! The private trailer-pairs helper is reached only through public squash; no production source inclusion/exposure.
use crate::support::{Fixture, CHANGE_ID};
use land::{LandRefs, SquashAnswer, SquashError};
use oplog::{KillPidProbe, LockName};

#[test]
fn rit_squash_writes_change_id_zone_and_squash_of_in_that_exact_order_without_a_request_trailer() {
    let f = Fixture::new();
    let refs = LandRefs { origin_branch: "main".into(), integration_branch: "integration".into(), remote: "origin".into() };
    let answer = oplog::with_lock(&f.git, &f.common, &LockName::Land, "land-rit-trailer-order", &KillPidProbe, || {
        oplog::recover(&f.git, &f.common).map_err(SquashError::Oplog)?;
        land::squash(&f.git, &refs, "core", "land-rit")
    }).unwrap();
    let SquashAnswer::Made(made) = answer else { panic!("fixture must write a new squash commit: {answer:?}") };
    assert_eq!(made.change_id.as_str(), CHANGE_ID); assert_eq!(made.source, f.source);
    let info = f.git.commit_info(&made.commit).unwrap();
    let expected = vec![
        format!("Mortar-Change-Id: {CHANGE_ID}"),
        "Mortar-Zone: core".to_string(),
        format!("Mortar-Squash-Of: {}", f.source),
    ];
    let written: Vec<&str> = info.message.lines().filter(|line| line.starts_with("Mortar-")).collect();
    assert_eq!(written, expected);
    assert!(info.message.ends_with(&format!("\n\n{}\n", expected.join("\n"))));
    // Read the actual persisted footer through the two lower kits, not the helper that produced its pairs.
    assert_eq!(f.git.trailers(&info.message).unwrap(), vec![
        ("Mortar-Change-Id".into(), CHANGE_ID.into()),
        ("Mortar-Zone".into(), "core".into()),
        ("Mortar-Squash-Of".into(), f.source.to_string()),
    ]);
    let read = oplog::read_mortar_trailers(&f.git, &info.message).unwrap();
    assert_eq!(read.change_id.unwrap().as_str(), CHANGE_ID); assert_eq!(read.zone.as_deref(), Some("core"));
    assert_eq!(read.request, None);
}
