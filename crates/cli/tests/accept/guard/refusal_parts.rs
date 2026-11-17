//! Reads the guard's refusal out of what a refused git command printed.

/// The refusal's why, owner and next, as `mortar guard-hook` prints them in plain text (one why line, then
/// "owner: <owner>", then "next: <next>") and git passes them on among its own lines: why is the line just before
/// the first "owner: " line, next the first "next: " line after it. All three are "" when no line starts with
/// "owner: ", and why or next is "" when its line is missing.
pub fn refusal_parts(output: &str) -> (String, String, String) {
    let lines: Vec<&str> = output.lines().map(str::trim_end).collect();
    let Some((at, owner)) = lines.iter().enumerate().find_map(|(at, line)| Some((at, line.strip_prefix("owner: ")?)))
    else {
        return (String::new(), String::new(), String::new());
    };
    let why = at.checked_sub(1).map_or("", |before| lines[before]);
    let next = lines[at + 1..].iter().find_map(|line| line.strip_prefix("next: ")).unwrap_or("");
    (why.to_string(), owner.to_string(), next.to_string())
}
