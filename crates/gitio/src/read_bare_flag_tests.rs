use crate::read_bare_flag::read_bare_flag;

#[test]
fn true_is_bare() {
    assert!(read_bare_flag(b"true\n").unwrap());
}

#[test]
fn false_is_not_bare() {
    assert!(!read_bare_flag(b"false\n").unwrap());
}

#[test]
fn any_other_answer_is_refused_naming_it() {
    let err = read_bare_flag(b"yes\n").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("\"yes\\n\""), "{}", err.stderr);
}
