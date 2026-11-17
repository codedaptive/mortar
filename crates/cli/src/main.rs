//! mortar: the binary under the name `mortar`, the same binary as `mr` (src/bin/mr.rs, S-133).
//! The body is cli::main_entry; the name it was run under changes nothing.

fn main() {
    std::process::exit(cli::main_entry())
}
