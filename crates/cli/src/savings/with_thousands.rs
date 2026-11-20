//! A number with thousands separators.

/// The decimal digits of `n` with a comma before each group of three counted from the right: 1234567 is
/// "1,234,567"; below 1000 there is no comma.
pub fn with_thousands(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, digit) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    out
}
