//! Whether a git call only reads (step 4's check 3; config and remote read only in the forms named below).

/// The subcommands that are reads whatever follows them (`version` is the git floor check every verb but the exempt
/// ones makes first); `config` and `remote` are reads only in the forms `git_call_is_read` names.
const READS: [&str; 10] =
    ["status", "diff", "log", "show", "ls-files", "rev-parse", "blame", "grep", "cat-file", "version"];

/// `args` is git's argv, argv[0] the program. The subcommand is the first argument after argv[0] that is not an
/// option: `-C <dir>` and `-c <key=value>` skip their value too, and any other argument starting with `-`
/// (`--no-pager`, `--git-dir=<dir>`) is skipped alone. True when that subcommand is one of the reads, or when there
/// is none (`git --version`, like `git version`, changes nothing). `config` is a read when it has a reading flag (`--get`, `--get-all`,
/// `--get-regexp`, `--list`, `-l`) and every other option is a scope (`--global`, `--system`, `--local`,
/// `--worktree`, `--file`/`-f <f>`, `--file=<f>`, `--blob <b>`, `--blob=<b>`), `--includes`, `-z` or `--null`.
/// `remote` is a read alone, with only `-v` or `--verbose`, or as `get-url` with one name and only `--push` or
/// `--all` (in any position). Every other form, of these two and of every other subcommand, is a write: only the
/// forms named here are reads, and a form not named here (`config <key>`, `remote show`) counts as a write.
pub fn git_call_is_read(args: &[String]) -> bool {
    let mut rest = args.iter();
    rest.next();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "-C" | "-c" => {
                rest.next();
            }
            option if option.starts_with('-') => {}
            "config" => return config_is_read(rest.as_slice()),
            "remote" => return remote_is_read(rest.as_slice()),
            subcommand => return READS.contains(&subcommand),
        }
    }
    true
}

/// `rest` is everything after `config`. Positionals (the name, a value pattern) are allowed; `--file`, `-f` and
/// `--blob` consume the next argument (`--file=`/`--blob=` carry their value), so a value that looks like a flag is not
/// taken for one. Any other option makes it false; without a reading flag it is false.
fn config_is_read(rest: &[String]) -> bool {
    let mut reading = false;
    let mut args = rest.iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--file" | "-f" | "--blob" => {
                args.next();
            }
            "--get" | "--get-all" | "--get-regexp" | "--list" | "-l" => reading = true,
            "--global" | "--system" | "--local" | "--worktree" | "--includes" | "-z" | "--null" => {}
            option if option.starts_with("--file=") || option.starts_with("--blob=") => {}
            option if option.starts_with('-') => return false,
            _ => {}
        }
    }
    reading
}

/// `rest` is everything after `remote`.
fn remote_is_read(rest: &[String]) -> bool {
    match rest {
        [] => true,
        [flag] => flag == "-v" || flag == "--verbose",
        [sub, after @ ..] => {
            sub == "get-url"
                && after.iter().all(|arg| arg == "--push" || arg == "--all" || !arg.starts_with('-'))
                && after.iter().filter(|arg| !arg.starts_with('-')).count() == 1
        }
    }
}
