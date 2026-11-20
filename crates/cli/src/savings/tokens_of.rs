//! A character count as an estimated token count.

/// Four characters to a token, rounded half up: the value of (chars + 2) / 4, computed from the quotient and
/// remainder so that u64::MAX does not overflow.
pub fn tokens_of(chars: u64) -> u64 {
    chars / 4 + u64::from(chars % 4 >= 2)
}
