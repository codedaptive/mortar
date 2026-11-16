use crate::read_sparse_flag::read_sparse_flag;
use crate::Output;

fn output(status: i32, stdout: &[u8]) -> Output {
    Output {
        status,
        stdout: stdout.to_vec(),
        stderr: Vec::new(),
    }
}

#[test]
fn exit_1_is_not_sparse() {
    assert!(!read_sparse_flag(&output(1, b"")).unwrap());
}

#[test]
fn exit_1_is_not_sparse_whatever_stdout_holds() {
    assert!(!read_sparse_flag(&output(1, b"true\n")).unwrap());
}

#[test]
fn true_is_sparse() {
    assert!(read_sparse_flag(&output(0, b"true\n")).unwrap());
}

#[test]
fn false_is_not_sparse() {
    assert!(!read_sparse_flag(&output(0, b"false\n")).unwrap());
}

#[test]
fn true_without_its_line_break_is_refused_naming_stdout() {
    let err = read_sparse_flag(&output(0, b"true")).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert!(err.stderr.contains("\"true\""), "{}", err.stderr);
}

#[test]
fn an_unparsed_spelling_is_refused_naming_stdout() {
    let err = read_sparse_flag(&output(0, b"yes\n")).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("\"yes\\n\""), "{}", err.stderr);
}

#[test]
fn empty_stdout_on_exit_0_is_refused() {
    let err = read_sparse_flag(&output(0, b"")).unwrap_err();
    assert!(err.stderr.contains("\"\""), "{}", err.stderr);
}

#[test]
fn non_utf8_stdout_is_refused_naming_it_lossily() {
    let err = read_sparse_flag(&output(0, b"tr\xffue\n")).unwrap_err();
    assert!(err.stderr.contains("\"tr\u{fffd}ue\\n\""), "{}", err.stderr);
}
