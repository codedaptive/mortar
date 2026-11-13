use crate::read_commit_info::read_commit_info;
use crate::{CommitInfo, Oid};

const ROOT: &str = "9145c5477bb7654747096c3188e5e006608d1c14";
const SECOND: &str = "6269f0e88f79b3eada64287692f6d3f03196c4a7";
const MERGE: &str = "726080b6b018f93a1bbce14e3537e998ff5f62bd";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn root_commit_has_no_parents_and_its_whole_message() {
    let out = b"9145c5477bb7654747096c3188e5e006608d1c14\x003be22be77da4887e869c981806d8452f034dd014\0\0first\n\nbody line\n\0";
    let want = CommitInfo {
        tree: oid("3be22be77da4887e869c981806d8452f034dd014"),
        parents: vec![],
        message: "first\n\nbody line\n".to_string(),
    };
    assert_eq!(read_commit_info(out, &oid(ROOT)), Ok(want));
}

#[test]
fn second_commit_has_one_parent() {
    let out = b"6269f0e88f79b3eada64287692f6d3f03196c4a7\x006cf76faefcefa1121f2a3e663edc331d21fe9797\x009145c5477bb7654747096c3188e5e006608d1c14\x00second\n\0";
    let want = CommitInfo {
        tree: oid("6cf76faefcefa1121f2a3e663edc331d21fe9797"),
        parents: vec![oid(ROOT)],
        message: "second\n".to_string(),
    };
    assert_eq!(read_commit_info(out, &oid(SECOND)), Ok(want));
}

#[test]
fn merge_commit_has_both_parents_in_order() {
    let out = b"726080b6b018f93a1bbce14e3537e998ff5f62bd\x0013c0f4b7e640943ffe76fbdd7f0f286dd1fbd14a\x006269f0e88f79b3eada64287692f6d3f03196c4a7 cf9b8986512445d26313ea6a44990dd9599e8d3c\x00Merge branch 'side'\n\0";
    let want = CommitInfo {
        tree: oid("13c0f4b7e640943ffe76fbdd7f0f286dd1fbd14a"),
        parents: vec![oid(SECOND), oid("cf9b8986512445d26313ea6a44990dd9599e8d3c")],
        message: "Merge branch 'side'\n".to_string(),
    };
    assert_eq!(read_commit_info(out, &oid(MERGE)), Ok(want));
}

#[test]
fn message_with_trailing_blank_lines_is_kept_unchanged() {
    let out = b"6269f0e88f79b3eada64287692f6d3f03196c4a7\x006cf76faefcefa1121f2a3e663edc331d21fe9797\x00\x00second\n\n\n\0";
    assert_eq!(read_commit_info(out, &oid(SECOND)).unwrap().message, "second\n\n\n");
}

#[test]
fn empty_stdout_is_not_a_commit_naming_the_id() {
    let err = read_commit_info(b"", &oid("3be22be77da4887e869c981806d8452f034dd014")).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert!(err.stderr.contains("not a commit"), "{}", err.stderr);
    assert!(err.stderr.contains("3be22be77da4887e869c981806d8452f034dd014"), "{}", err.stderr);
}

#[test]
fn missing_record_nul_is_refused_naming_the_bytes() {
    let out = b"6269f0e88f79b3eada64287692f6d3f03196c4a7\x006cf76faefcefa1121f2a3e663edc331d21fe9797\x00\x00second\n";
    let err = read_commit_info(out, &oid(SECOND)).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("second\\n\""), "{}", err.stderr);
}

#[test]
fn fewer_than_four_fields_is_refused_naming_the_bytes() {
    let out = b"6269f0e88f79b3eada64287692f6d3f03196c4a7\x006cf76faefcefa1121f2a3e663edc331d21fe9797\0";
    let err = read_commit_info(out, &oid(SECOND)).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("6cf76faefcefa1121f2a3e663edc331d21fe9797\\0\""), "{}", err.stderr);
}

#[test]
fn id_differing_from_the_commit_asked_is_refused() {
    let out = b"6269f0e88f79b3eada64287692f6d3f03196c4a7\x006cf76faefcefa1121f2a3e663edc331d21fe9797\x00\x00second\n\0";
    let err = read_commit_info(out, &oid(ROOT)).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains(ROOT), "{}", err.stderr);
    assert!(err.stderr.contains("\"6269f0e88f79b3eada64287692f6d3f03196c4a7\\0"), "{}", err.stderr);
}

#[test]
fn bad_tree_id_is_refused_naming_it() {
    let out = b"6269f0e88f79b3eada64287692f6d3f03196c4a7\x006CF76FAE\x00\x00second\n\0";
    let err = read_commit_info(out, &oid(SECOND)).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("\"6CF76FAE\""), "{}", err.stderr);
}

#[test]
fn bad_parent_id_is_refused_naming_it() {
    let out = b"6269f0e88f79b3eada64287692f6d3f03196c4a7\x006cf76faefcefa1121f2a3e663edc331d21fe9797\x009145c54\x00second\n\0";
    let err = read_commit_info(out, &oid(SECOND)).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("\"9145c54\""), "{}", err.stderr);
}

#[test]
fn double_space_between_parents_is_refused() {
    let out = b"726080b6b018f93a1bbce14e3537e998ff5f62bd\x0013c0f4b7e640943ffe76fbdd7f0f286dd1fbd14a\x006269f0e88f79b3eada64287692f6d3f03196c4a7  cf9b8986512445d26313ea6a44990dd9599e8d3c\x00Merge branch 'side'\n\0";
    let err = read_commit_info(out, &oid(MERGE)).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("\"\""), "{}", err.stderr);
}

#[test]
fn non_utf8_message_is_refused_naming_it() {
    let out = b"6269f0e88f79b3eada64287692f6d3f03196c4a7\x006cf76faefcefa1121f2a3e663edc331d21fe9797\x00\x00caf\xe9\n\0";
    let err = read_commit_info(out, &oid(SECOND)).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("\"caf\u{fffd}\\n\""), "{}", err.stderr);
}
