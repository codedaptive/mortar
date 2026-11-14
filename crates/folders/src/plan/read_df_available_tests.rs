//! `df -P -k` output as macOS and Linux print it, and each refusal.

use super::read_df_available;

#[test]
fn reads_macos_output() {
    let out = b"Filesystem     1024-blocks      Used Available Capacity  Mounted on\n\
/dev/disk3s5     971350180 600000000 350000000    64%    /System/Volumes/Data\n";
    assert_eq!(read_df_available(out), Ok(358_400_000_000));
}

#[test]
fn reads_linux_output() {
    let out = b"Filesystem     1024-blocks     Used Available Capacity Mounted on\n\
/dev/nvme0n1p2   479151816 98765432 355999999      22% /\n";
    assert_eq!(read_df_available(out), Ok(364_543_998_976));
}

#[test]
fn reads_without_a_trailing_newline() {
    let out = b"Filesystem 1024-blocks Used Available Capacity Mounted on\ntmpfs 100 90 10 90% /tmp";
    assert_eq!(read_df_available(out), Ok(10_240));
}

#[test]
fn reads_zero_available() {
    let out = b"Filesystem 1024-blocks Used Available Capacity Mounted on\n/dev/sda1 100 100 0 100% /\n";
    assert_eq!(read_df_available(out), Ok(0));
}

#[test]
fn refuses_header_only() {
    let out = b"Filesystem 1024-blocks Used Available Capacity Mounted on\n";
    assert!(read_df_available(out).unwrap_err().starts_with("df printed no data line"));
}

#[test]
fn refuses_empty_output() {
    assert!(read_df_available(b"").unwrap_err().starts_with("df printed no data line"));
}

#[test]
fn refuses_a_short_data_line() {
    let out = b"Filesystem 1024-blocks Used Available Capacity Mounted on\n/dev/sda1 100 90\n";
    assert!(read_df_available(out).unwrap_err().starts_with("df data line has no available column"));
}

#[test]
fn refuses_a_non_numeric_column() {
    let out = b"Filesystem 1024-blocks Used Available Capacity Mounted on\n/dev/sda1 100 90 ten 90% /\n";
    assert_eq!(read_df_available(out), Err("df available column \"ten\" is not a number".to_string()));
}

#[test]
fn accepts_the_largest_block_count_that_fits() {
    let out = b"Filesystem 1024-blocks Used Available Capacity Mounted on\nbig 1 1 18014398509481983 1% /\n";
    assert_eq!(read_df_available(out), Ok(18_446_744_073_709_550_592));
}

#[test]
fn refuses_overflow() {
    let out = b"Filesystem 1024-blocks Used Available Capacity Mounted on\nbig 1 1 18014398509481984 1% /\n";
    assert_eq!(
        read_df_available(out),
        Err("df available blocks 18014398509481984 overflow a byte count".to_string())
    );
}
