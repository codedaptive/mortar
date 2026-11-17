//! Reads what an `mr ... --json` run printed.

use super::Reply;
use crate::json::{parse_json, JsonValue};
use crate::world::Ran;

/// The report on `ran`'s stdout, read with crate::json. `saw` is the run's ending (Ran::seen), then "; outcome
/// <outcome>", then, when the exit is not 0 and the report has lines, "; " and its first line. Stdout that is not
/// JSON gives an empty outcome, next and lines, a Null report, and a `saw` naming the parse error.
pub fn read_reply(ran: &Ran) -> Reply {
    let ended = ran.seen();
    let mut reply = Reply {
        exit: ran.code,
        outcome: String::new(),
        next: String::new(),
        lines: Vec::new(),
        report: JsonValue::Null,
        saw: String::new(),
    };
    match parse_json(&ran.stdout) {
        Ok(report) => reply.report = report,
        Err(error) => {
            reply.saw = format!("{ended}; stdout is not JSON: {error}");
            return reply;
        }
    }
    reply.outcome = reply.text("outcome");
    reply.next = reply.text("next");
    reply.lines = reply.texts("lines");
    reply.saw = match (ran.code, reply.lines.first()) {
        (Some(0), _) | (_, None) => format!("{ended}; outcome {}", reply.outcome),
        (_, Some(first)) => format!("{ended}; outcome {}; {first}", reply.outcome),
    };
    reply
}
