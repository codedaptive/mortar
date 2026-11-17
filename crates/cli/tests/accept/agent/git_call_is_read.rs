//! Whether a git call only reads (04-agent.md, check 3).

/// The subcommands check 3 counts as reads.
const READS: [&str; 9] = ["status", "diff", "log", "show", "ls-files", "rev-parse", "blame", "grep", "cat-file"];

/// `args` is git's argv, argv[0] the program. The subcommand is the first argument after argv[0] that is not an
/// option: `-C <dir>` and `-c <key=value>` skip their value too, and any other argument starting with `-`
/// (`--no-pager`, `--git-dir=<dir>`) is skipped alone. True when that subcommand is one of the reads, or when there
/// is none (`git --version` changes nothing); false for every other subcommand.
pub fn git_call_is_read(args: &[String]) -> bool {
    let mut rest = args.iter().skip(1);
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "-C" | "-c" => {
                rest.next();
            }
            option if option.starts_with('-') => {}
            subcommand => return READS.contains(&subcommand),
        }
    }
    true
}
