//! mortar: the `mortar` binary; src/bin/mr.rs is the same binary under `mr` (S-133).
//! The body is cli::main_entry; the name it was run under changes nothing.

fn main() {
    std::process::exit(cli::main_entry())
}
