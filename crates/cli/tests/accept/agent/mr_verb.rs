//! The verb an `mr` or `mortar` call names.

/// `args` is the argv, argv[0] the program. The first argument after argv[0] that does not start with `-` and is
/// not the value of `--key` or `--actor` (the word after either is skipped; `--key=<v>` and `--actor=<v>` start
/// with `-`); None when there is none (`mr --help`, a bare `mr`).
pub fn mr_verb(args: &[String]) -> Option<&str> {
    let mut read = args.iter().skip(1).map(String::as_str);
    while let Some(arg) = read.next() {
        match arg {
            "--key" | "--actor" => {
                read.next();
            }
            _ if arg.starts_with('-') => {}
            _ => return Some(arg),
        }
    }
    None
}
