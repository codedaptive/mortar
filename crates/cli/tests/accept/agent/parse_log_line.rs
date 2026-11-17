//! Reads one line of mr.log or git.log.

use super::LogCall;

/// A line is tab-separated: pid, parent pid, the parent's command base name, then the argv (at least argv[0]).
/// None when a pid is not a number, the parent is empty or there is no argv.
pub fn parse_log_line(line: &str) -> Option<LogCall> {
    let mut fields = line.split('\t');
    let pid = fields.next()?.parse().ok()?;
    let ppid = fields.next()?.parse().ok()?;
    let parent = fields.next().filter(|parent| !parent.is_empty())?.to_string();
    let args: Vec<String> = fields.map(str::to_string).collect();
    if args.is_empty() {
        return None;
    }
    Some(LogCall { pid, ppid, parent, args })
}
