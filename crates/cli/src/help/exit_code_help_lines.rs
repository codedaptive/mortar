//! exit_code_help_lines: the program help's block on exit codes, between the groups and the lines about tokens saved.

/// An empty line, the heading "Exit codes", one row per ExitCode in numeric order ("  <code>  <word>: <meaning>"),
/// then which code a run answers when several could apply. The order is run_in's: parse_args first, then git_floor
/// and shell_context (folder, actor, config), then the verb, whose own argument checks come before its locked_act.
pub fn exit_code_help_lines() -> Vec<String> {
    [
        "",
        "Exit codes",
        "  0  done: the command did its work, or found it already done",
        "  1  conflict: the work conflicted; the report lists the conflicted files",
        "  2  usage or configuration: the command line could not be read, the run is outside a git repository, or the config or guard.json it needs is missing or cannot be read; the report names the fix",
        "  3  refused: the report names who owns the fix and the next step",
        "  4  busy: a lock is held or a ref moved mid-run; repeat it later, or first take the step the report names",
        "  5  failed: git or the disk failed; the report names the next step",
        "Each run answers one code, from the first check that stops it: the command line is read first (2), then git's",
        "version, the folder, the actor and the config (2, 3 or 5); then the verb checks its own arguments (2) and what",
        "it acts on (3 or 5; a config or guard.json it cannot read is 2) before it takes any lock (4); the work itself",
        "ends in any code but 2, except mr guard status, which reads guard.json under its lock, and guard-hook, which",
        "reads git's common dir and guard.json before its own words.",
    ]
    .map(str::to_string)
    .to_vec()
}
