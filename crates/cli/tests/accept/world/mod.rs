//! A step's throwaway repositories, their bin folder of `mr` and `mortar`, and the runners for git, `mr` and a
//! killed program inside them.

mod crash;
mod crash_as_str;
mod ran;
mod ran_from_error;
mod ran_from_output;
mod ran_seen;
mod scratch;
mod scratch_close;
mod scratch_drop;
mod scratch_new;
mod scratch_path;
mod world;
mod world_bin_dir;
mod world_close;
mod world_command;
mod world_env;
mod world_git;
mod world_killed;
mod world_mortar;
mod world_mortar_bin;
mod world_mr_bin;
mod world_new;

#[cfg(test)]
mod ran_from_error_tests;
#[cfg(test)]
mod ran_seen_tests;
#[cfg(test)]
mod world_bin_dir_tests;
#[cfg(test)]
mod world_env_tests;

pub use crash::Crash;
pub use ran::Ran;
pub use scratch::Scratch;
pub use world::World;
pub use world_bin_dir::world_bin_dir;
pub use world_env::world_env;
