use crate::read_oid_lines::read_oid_lines;
use crate::Oid;

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn empty_output_is_no_ids() {
    assert_eq!(read_oid_lines(b""), Ok(vec![]));
}

#[test]
fn rev_list_capture_keeps_git_order() {
    let out = b"6269f0e88f79b3eada64287692f6d3f03196c4a7\ncf9b8986512445d26313ea6a44990dd9599e8d3c\n726080b6b018f93a1bbce14e3537e998ff5f62bd\n1e00b160b08b2c3aa37f5f0806e78e7f627a1ebf\n";
    let want = vec![
        oid("6269f0e88f79b3eada64287692f6d3f03196c4a7"),
        oid("cf9b8986512445d26313ea6a44990dd9599e8d3c"),
        oid("726080b6b018f93a1bbce14e3537e998ff5f62bd"),
        oid("1e00b160b08b2c3aa37f5f0806e78e7f627a1ebf"),
    ];
    assert_eq!(read_oid_lines(out), Ok(want));
}

#[test]
fn bad_id_is_refused_naming_it() {
    let err = read_oid_lines(b"6269f0e88f79b3eada64287692f6d3f03196c4a7\nnot-an-id\n").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert!(err.stderr.contains("\"not-an-id\""), "{}", err.stderr);
}

#[test]
fn empty_line_is_refused() {
    let err = read_oid_lines(b"6269f0e88f79b3eada64287692f6d3f03196c4a7\n\ncf9b8986512445d26313ea6a44990dd9599e8d3c\n")
        .unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("\"\""), "{}", err.stderr);
}

#[test]
fn missing_final_newline_is_refused_naming_the_text() {
    let err = read_oid_lines(b"6269f0e88f79b3eada64287692f6d3f03196c4a7").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("\"6269f0e88f79b3eada64287692f6d3f03196c4a7\""), "{}", err.stderr);
}
