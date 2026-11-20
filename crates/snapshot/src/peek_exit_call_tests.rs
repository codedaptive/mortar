use crate::peek_exit_call::peek_exit_call;
use crate::waitid_args::WaitidArgs;

#[test]
fn linux_peeks_by_pid_with_wexited_wnohang_and_its_wnowait() {
    assert_eq!(peek_exit_call("linux"), Some(WaitidArgs { idtype: 1, options: 0x0100_0005 }));
}

#[test]
fn macos_peeks_by_pid_with_wexited_wnohang_and_its_wnowait() {
    assert_eq!(peek_exit_call("macos"), Some(WaitidArgs { idtype: 1, options: 0x25 }));
}

#[test]
fn freebsd_has_no_peek() {
    assert_eq!(peek_exit_call("freebsd"), None);
}

#[test]
fn windows_has_no_peek() {
    assert_eq!(peek_exit_call("windows"), None);
}

#[test]
fn an_empty_os_name_has_no_peek() {
    assert_eq!(peek_exit_call(""), None);
}
