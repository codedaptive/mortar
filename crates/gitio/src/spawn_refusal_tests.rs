use crate::spawn_refusal::spawn_refusal;
use std::io;
use std::path::Path;

#[test]
fn names_the_program_and_the_io_error_with_no_code() {
    let err = io::Error::new(io::ErrorKind::NotFound, "no such file");
    let refusal = spawn_refusal(Path::new("/usr/bin/git"), &["status".to_string()], &err);
    assert_eq!(refusal.code, None);
    assert_eq!(refusal.args, vec!["status".to_string()]);
    assert_eq!(refusal.stderr, "could not run /usr/bin/git: no such file");
}

#[test]
fn carries_a_permission_error_text() {
    let err = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
    let refusal = spawn_refusal(Path::new("git"), &[], &err);
    assert_eq!(refusal.stderr, "could not run git: permission denied");
    assert!(refusal.args.is_empty());
}
