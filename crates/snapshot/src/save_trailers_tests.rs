use crate::save_trailers::save_trailers;
use crate::SnapshotTarget;
use oplog::ChangeId;

#[test]
fn change_id_then_request_then_zone() {
    let target = SnapshotTarget {
        zone: "snapshot".to_string(),
        request: "r3".to_string(),
        attempt: None,
        actor: "agent-a".to_string(),
    };
    let change_id = ChangeId::parse("kkkkllllmmmmnnnnooooppppqqqqrrrr").unwrap();
    assert_eq!(
        save_trailers(&change_id, &target),
        vec![
            ("Mortar-Change-Id".to_string(), "kkkkllllmmmmnnnnooooppppqqqqrrrr".to_string()),
            ("Mortar-Request".to_string(), "r3".to_string()),
            ("Mortar-Zone".to_string(), "snapshot".to_string()),
        ]
    );
}
