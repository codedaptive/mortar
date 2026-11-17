//! mr: the binary under the name `mr` (S-133); `mortar` (src/main.rs) is the other name for the same binary.
//! The body is cli::main_entry; the name it was run under changes nothing.

fn main() {
    std::process::exit(cli::main_entry())
}
