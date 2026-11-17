use super::lock_pid;

#[test]
fn reads_the_pid_line_of_a_lock_record() {
    assert_eq!(lock_pid("pid=4242\nhost=box\nkey=join:/tmp/r1\nsince=1700000000\n"), Some(4242));
}

#[test]
fn the_pid_line_need_not_come_first() {
    assert_eq!(lock_pid("host=box\npid=17\n"), Some(17));
}

#[test]
fn no_pid_line_or_a_value_that_is_not_a_pid_is_none() {
    assert_eq!(lock_pid("host=box\nkey=k\n"), None);
    assert_eq!(lock_pid("pid=abc\n"), None);
    assert_eq!(lock_pid("pid=-3\n"), None);
    assert_eq!(lock_pid(""), None);
}

#[test]
fn a_key_holding_pid_text_is_not_read_as_the_pid() {
    assert_eq!(lock_pid("key=pid=9\nhost=box\n"), None);
}
