//! main_entry: the body of the binary under both its names (`mortar` and `mr`): reads the arguments and the
//! current directory, runs the verb, prints its report unless it prints nothing, and returns the exit code.
//! No sibling unit test: it reads the process's arguments and directory and prints (S-5, S-104); cli's unit tests
//! cover the reports. Nagatha's integration group is to cover, with the binary as a child process in a throwaway
//! repository: `mortar init --actor h --origin main --integration integration --remote origin` writes the
//! config, exit 0, stdout ending "next: mortar folder zone <zone> <path>"; the same again is exit 0
//! "already-done" with the file byte-identical; other values are exit 1 naming the keys; --replace rewrites,
//! exit 0; init without --actor is exit 4; an unknown verb is exit 4 listing the verbs; --json prints one JSON
//! object on one line and nothing else; run outside a git repository init is refused, exit 1; a non-UTF-8
//! argument is exit 4; a held lock is exit 3 naming the holder and since, and recover runs before the act (with
//! mortar join, the first ref-moving verb); an agent verb outside a Mortar folder is refused, exit 1 (with
//! mortar join, the first agent verb); `mortar guard-hook` allowing a move prints nothing on stdout or stderr,
//! exit 0, also with --json (S-111).

use std::ffi::OsString;
use std::io::Write;

use crate::{failure_report, prints_nothing, render_json, render_plain, run, usage_report, UsageError};

pub fn main_entry() -> i32 {
    let words = std::env::args_os().skip(1).collect::<Vec<OsString>>();
    // Read before anything else, so that a word that is not UTF-8 is reported in the form asked for.
    let json = words.iter().take_while(|word| *word != "--").any(|word| word == "--json");
    let argv = words.iter().map(|word| word.clone().into_string()).collect::<Result<Vec<String>, OsString>>();
    let (report, json) = match (argv, std::env::current_dir()) {
        (Err(word), _) => {
            let reason = format!("the argument {:?} is not UTF-8", word.to_string_lossy());
            (usage_report(&UsageError { verb: None, reason, json }), json)
        }
        (Ok(_), Err(error)) => (failure_report("", &format!("cannot read the current directory: {error}")), json),
        (Ok(argv), Ok(cwd)) => run(&argv, &cwd),
    };
    if !prints_nothing(&report) {
        let text = if json { render_json(&report) } else { render_plain(&report) };
        // Nothing is left to tell anyone when stdout itself cannot be written; the exit code still says the outcome.
        let mut stdout = std::io::stdout().lock();
        let _ = stdout.write_all(text.as_bytes()).and_then(|()| stdout.flush());
    }
    report.exit as i32
}
