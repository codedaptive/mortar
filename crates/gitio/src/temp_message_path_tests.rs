use crate::temp_message_path::temp_message_path;
use std::path::Path;

#[test]
fn name_holds_pid_stamp_and_serial_under_the_dir() {
    assert_eq!(
        temp_message_path(Path::new("/tmp"), 4321, 1_700_000_000_123_456_789, 7),
        Path::new("/tmp/mortar-message-4321-1700000000123456789-7")
    );
}

#[test]
fn another_serial_is_another_path() {
    assert_ne!(
        temp_message_path(Path::new("/var/t"), 1, 2, 0),
        temp_message_path(Path::new("/var/t"), 1, 2, 1)
    );
}

#[test]
fn another_pid_or_stamp_is_another_path() {
    let one = temp_message_path(Path::new("/var/t"), 1, 2, 0);
    assert_ne!(one, temp_message_path(Path::new("/var/t"), 9, 2, 0));
    assert_ne!(one, temp_message_path(Path::new("/var/t"), 1, 3, 0));
}
