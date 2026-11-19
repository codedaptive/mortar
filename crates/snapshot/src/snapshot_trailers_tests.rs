use crate::snapshot_trailers::snapshot_trailers;
use crate::SnapshotTarget;

#[test]
fn request_then_zone() {
    let target = SnapshotTarget {
        zone: "snapshot".to_string(),
        request: "r1".to_string(),
        attempt: None,
        actor: "agent-a".to_string(),
    };
    assert_eq!(
        snapshot_trailers(&target),
        vec![
            ("Mortar-Request".to_string(), "r1".to_string()),
            ("Mortar-Zone".to_string(), "snapshot".to_string()),
        ]
    );
}
