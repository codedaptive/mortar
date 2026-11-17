//! The verb an `mr` or `mortar` call names.

/// `args` is the argv, argv[0] the program. The first argument after argv[0] that does not start with `-`; None
/// when there is none (`mr --help`).
pub fn mr_verb(args: &[String]) -> Option<&str> {
    args.iter().skip(1).find(|arg| !arg.starts_with('-')).map(String::as_str)
}
