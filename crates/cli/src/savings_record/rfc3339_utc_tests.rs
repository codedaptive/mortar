//! rfc3339_utc: verified timestamps, leap days and year ends included.

use super::rfc3339_utc;

#[test]
fn the_verified_literals() {
    let cases = [
        (0, "1970-01-01T00:00:00Z"),
        (951_782_400, "2000-02-29T00:00:00Z"),
        (1_700_000_000, "2023-11-14T22:13:20Z"),
        (1_798_761_599, "2026-12-31T23:59:59Z"),
        (4_102_444_800, "2100-01-01T00:00:00Z"),
        (1_763_606_400, "2025-11-20T02:40:00Z"),
    ];
    for (secs, text) in cases {
        assert_eq!(rfc3339_utc(secs), text, "{secs}");
    }
}

#[test]
fn the_last_second_of_a_day_and_the_next() {
    // 86,399 s is 23:59:59 on the first day; 86,400 s is midnight of the second.
    assert_eq!(rfc3339_utc(86_399), "1970-01-01T23:59:59Z");
    assert_eq!(rfc3339_utc(86_400), "1970-01-02T00:00:00Z");
}
