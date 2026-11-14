use crate::snapshot_excludes::snapshot_excludes;

#[test]
fn no_extra_gives_the_six_built_ins_in_order() {
    assert_eq!(
        snapshot_excludes(&[]),
        vec!["target/", "*/target/*", "node_modules/", "*/node_modules/*", ".build/", "*/.build/*"]
    );
}

#[test]
fn extra_is_appended_in_order() {
    let extra = vec!["dist/".to_string(), "*.log".to_string()];
    assert_eq!(
        snapshot_excludes(&extra),
        vec![
            "target/", "*/target/*", "node_modules/", "*/node_modules/*", ".build/", "*/.build/*", "dist/", "*.log"
        ]
    );
}

#[test]
fn a_built_in_or_repeated_extra_appears_once() {
    let extra = vec!["node_modules/".to_string(), "dist/".to_string(), "dist/".to_string()];
    assert_eq!(
        snapshot_excludes(&extra),
        vec!["target/", "*/target/*", "node_modules/", "*/node_modules/*", ".build/", "*/.build/*", "dist/"]
    );
}
