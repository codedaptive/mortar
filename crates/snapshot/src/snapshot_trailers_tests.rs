use crate::snapshot_trailers::snapshot_trailers;
use crate::SnapshotTarget;

#[test]
fn request_then_zone() {
    let target = SnapshotTarget {
        zone: "snapshot".to_string(),
        request: "WO-snapshot-1".to_string(),
        attempt: None,
        actor: "bilby".to_string(),
    };
    assert_eq!(
        snapshot_trailers(&target),
        vec![
            ("Mortar-Request".to_string(), "WO-snapshot-1".to_string()),
            ("Mortar-Zone".to_string(), "snapshot".to_string()),
        ]
    );
}
