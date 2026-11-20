use super::millis_text;
use std::time::Duration;

#[test]
fn formats_milliseconds_to_one_decimal() {
    assert_eq!(millis_text(Duration::ZERO), "0.0 ms");
    assert_eq!(millis_text(Duration::from_millis(5)), "5.0 ms");
    assert_eq!(millis_text(Duration::from_micros(1_234_567)), "1234.6 ms");
    assert_eq!(millis_text(Duration::from_micros(240)), "0.2 ms");
    assert_eq!(millis_text(Duration::from_micros(260)), "0.3 ms");
}
