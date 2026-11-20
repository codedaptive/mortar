use super::byte_text;

#[test]
fn formats_apparent_bytes_and_rounded_mib() {
    assert_eq!(byte_text(0), "0 bytes (0.0 MiB)");
    assert_eq!(byte_text(1_048_576), "1048576 bytes (1.0 MiB)");
    assert_eq!(byte_text(1_572_864), "1572864 bytes (1.5 MiB)");
    assert_eq!(byte_text(123_456_789), "123456789 bytes (117.7 MiB)");
}
