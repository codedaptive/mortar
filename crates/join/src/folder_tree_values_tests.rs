use crate::folder_tree_values::folder_tree_values;
use gitio::Oid;

#[test]
fn a_folder_tree_is_one_pair_named_folder_tree() {
    let tree = Oid::parse("0123456789abcdef0123456789abcdef01234567").unwrap();
    assert_eq!(
        folder_tree_values(Some(&tree)),
        Ok(vec![("folder_tree".to_string(), "0123456789abcdef0123456789abcdef01234567".to_string())])
    );
}

#[test]
fn no_folder_tree_is_refused_naming_rebased_to_and_folder_tree() {
    let reason = folder_tree_values(None).unwrap_err();
    assert!(reason.contains("rebased_to"), "{reason}");
    assert!(reason.contains("folder_tree"), "{reason}");
}
