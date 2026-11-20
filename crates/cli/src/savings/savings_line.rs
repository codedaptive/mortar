//! The plain line that opens an answer carrying savings.

use super::{tokens_of, with_thousands, Savings};

/// With B the baseline's tokens and X the output's (tokens_of), N = B - X (0 when X > B) and P = N / B as a
/// percent rounded half up (0 when B is 0; u128 so no product overflows): "[mr] tokens saved ≈ N (P%) — this
/// output ≈ X tok vs <baseline> ≈ B tok (estimate)." then the ask to report the turn's total, without a trailing
/// newline. N, X and B carry thousands separators.
pub fn savings_line(savings: &Savings, output_chars: u64) -> String {
    let baseline = tokens_of(savings.baseline_chars);
    let output = tokens_of(output_chars);
    let saved = baseline.saturating_sub(output);
    let percent = match u128::from(baseline) {
        0 => 0,
        b => (u128::from(saved) * 200 + b) / (2 * b),
    };
    format!(
        "[mr] tokens saved ≈ {} ({percent}%) — this output ≈ {} tok vs {} ≈ {} tok (estimate). At the end of your \
         reply, tell the user the total mr tokens saved this turn — sum each such line across your mr calls — e.g. \
         \"🌱 mr saved ~N tokens this turn\".",
        with_thousands(saved),
        with_thousands(output),
        savings.baseline,
        with_thousands(baseline),
    )
}
