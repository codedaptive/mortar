//! The --json object of an answer's savings.

use oplog::write_json_string;

use super::Savings;

/// Appends {"baseline":<words>,"baselineChars":<baseline chars>,"outputChars":<output_chars>}: counts in
/// characters, not tokens, keys in that order.
pub fn write_saved_json(out: &mut String, savings: &Savings, output_chars: u64) {
    out.push_str("{\"baseline\":");
    write_json_string(out, &savings.baseline);
    out.push_str(&format!(",\"baselineChars\":{},\"outputChars\":{output_chars}}}", savings.baseline_chars));
}
