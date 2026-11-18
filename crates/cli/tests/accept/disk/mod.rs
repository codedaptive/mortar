//! Step 5, disk (05-disk.md): the refusal of a copy that is a registered team repository, the plan read from the
//! copy's HEAD, the folders Mortar makes from the copy under the scratch, the five numbers and their check lines;
//! run_disk is the body.

mod actor_mr;
mod byte_text;
mod checkout_plan;
mod copy_refusal;
mod disk_folders;
mod disk_number_lines;
mod disk_numbers;
mod git_common_dir;
mod manifest_repos;
mod millis_text;
mod parse_ls_tree;
mod plan_checkout;
mod registered;
mod registered_repos;
mod reply_number;
mod run_disk;
mod teams_folder;
mod time_snapshot;
mod tree_bytes;
mod tree_file;
mod writable_copy;

#[cfg(test)]
mod byte_text_tests;
#[cfg(test)]
mod copy_refusal_tests;
#[cfg(test)]
mod disk_number_lines_tests;
#[cfg(test)]
mod manifest_repos_tests;
#[cfg(test)]
mod millis_text_tests;
#[cfg(test)]
mod parse_ls_tree_tests;
#[cfg(test)]
mod plan_checkout_tests;
#[cfg(test)]
mod reply_number_tests;
#[cfg(test)]
mod teams_folder_tests;

pub use actor_mr::actor_mr;
pub use byte_text::byte_text;
pub use checkout_plan::CheckoutPlan;
pub use copy_refusal::copy_refusal;
pub use disk_folders::disk_folders;
pub use disk_number_lines::disk_number_lines;
pub use disk_numbers::DiskNumbers;
pub use git_common_dir::git_common_dir;
pub use manifest_repos::manifest_repos;
pub use millis_text::millis_text;
pub use parse_ls_tree::parse_ls_tree;
pub use plan_checkout::plan_checkout;
pub use registered::Registered;
pub use registered_repos::registered_repos;
pub use reply_number::reply_number;
pub use run_disk::run_disk;
pub use teams_folder::teams_folder;
pub use time_snapshot::time_snapshot;
pub use tree_bytes::tree_bytes;
pub use tree_file::TreeFile;
pub use writable_copy::writable_copy;
