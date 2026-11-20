//! with_thousands: a comma before each group of three digits from the right.

use super::with_thousands;

#[test]
fn the_contract_values() {
    let cases = [
        (0, "0"),
        (999, "999"),
        (1000, "1,000"),
        (526_660, "526,660"),
        (1_234_567, "1,234,567"),
        (u64::MAX, "18,446,744,073,709,551,615"),
    ];
    for (n, text) in cases {
        assert_eq!(with_thousands(n), text);
    }
}

#[test]
fn group_edges() {
    assert_eq!(with_thousands(100_000), "100,000");
    assert_eq!(with_thousands(10_000), "10,000");
    assert_eq!(with_thousands(1_000_000), "1,000,000");
}
