use crate::read_worktree_list::read_worktree_list;
use crate::{Oid, Worktree};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn worktree(path: &str) -> Worktree {
    Worktree {
        path: PathBuf::from(path),
        head: None,
        branch: None,
        bare: false,
        detached: false,
        locked: false,
        prunable: false,
    }
}

#[test]
fn captured_list_gives_a_branch_worktree_and_a_detached_locked_one() {
    let stdout = b"worktree /private/tmp/c/r\0HEAD 489d82745705a031306253da29bdad7aee19d2e5\0branch refs/heads/master\0\0worktree /private/tmp/c/wt2\0HEAD eec6b393124bfae85144b40fae63cde2d40c6430\0detached\0locked why not\0\0";
    assert_eq!(
        read_worktree_list(stdout),
        Ok(vec![
            Worktree {
                head: Some(oid("489d82745705a031306253da29bdad7aee19d2e5")),
                branch: Some("refs/heads/master".to_string()),
                ..worktree("/private/tmp/c/r")
            },
            Worktree {
                head: Some(oid("eec6b393124bfae85144b40fae63cde2d40c6430")),
                detached: true,
                locked: true,
                ..worktree("/private/tmp/c/wt2")
            },
        ])
    );
}

#[test]
fn empty_output_is_no_worktrees() {
    assert_eq!(read_worktree_list(b""), Ok(vec![]));
}

#[test]
fn bare_record_has_no_head() {
    assert_eq!(
        read_worktree_list(b"worktree /srv/r.git\0bare\0\0"),
        Ok(vec![Worktree { bare: true, ..worktree("/srv/r.git") }])
    );
}

#[test]
fn prunable_with_reason_and_locked_without_one() {
    let stdout = b"worktree /tmp/gone\0HEAD 0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\0detached\0locked\0prunable gitdir file points to non-existent location\0\0";
    assert_eq!(
        read_worktree_list(stdout),
        Ok(vec![Worktree {
            head: Some(oid("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")),
            detached: true,
            locked: true,
            prunable: true,
            ..worktree("/tmp/gone")
        }])
    );
}

#[test]
fn unknown_attributes_are_skipped() {
    let stdout = b"worktree /tmp/w\0HEAD 489d82745705a031306253da29bdad7aee19d2e5\0future-thing x y\0branch refs/heads/w\0\0";
    assert_eq!(
        read_worktree_list(stdout),
        Ok(vec![Worktree {
            head: Some(oid("489d82745705a031306253da29bdad7aee19d2e5")),
            branch: Some("refs/heads/w".to_string()),
            ..worktree("/tmp/w")
        }])
    );
}

#[test]
fn non_utf8_path_with_spaces_is_kept_as_its_bytes() {
    let list = read_worktree_list(b"worktree /tmp/caf\xe9 two words\0bare\0\0").unwrap();
    assert_eq!(list[0].path.as_os_str().as_bytes(), b"/tmp/caf\xe9 two words");
}

#[test]
fn record_not_starting_with_its_path_is_refused_naming_it() {
    let err = read_worktree_list(b"HEAD 489d82745705a031306253da29bdad7aee19d2e5\0\0").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert!(err.stderr.contains("\"HEAD 489d82745705a031306253da29bdad7aee19d2e5\""), "{}", err.stderr);
}

#[test]
fn empty_path_is_refused() {
    let err = read_worktree_list(b"worktree \0bare\0\0").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("\"worktree \""), "{}", err.stderr);
}

#[test]
fn bad_head_is_refused_naming_it() {
    let err = read_worktree_list(b"worktree /tmp/w\0HEAD 489d827\0\0").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("\"489d827\""), "{}", err.stderr);
}

#[test]
fn branch_without_a_name_is_refused() {
    let err = read_worktree_list(b"worktree /tmp/w\0branch\0\0").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("\"branch\""), "{}", err.stderr);
}

#[test]
fn non_utf8_branch_is_refused() {
    let err = read_worktree_list(b"worktree /tmp/w\0branch refs/heads/caf\xe9\0\0").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("not UTF-8"), "{}", err.stderr);
}

#[test]
fn record_without_its_closing_empty_field_is_refused() {
    let err = read_worktree_list(b"worktree /tmp/w\0bare\0").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("not ended by an empty field"), "{}", err.stderr);
}

#[test]
fn field_without_its_nul_is_refused() {
    let err = read_worktree_list(b"worktree /tmp/w\0\0worktree /tmp/x").unwrap_err();
    assert!(err.stderr.contains("not ended by an empty field"), "{}", err.stderr);
}
