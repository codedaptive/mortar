//! Zone integration tests of land::squash, land::would_merge and land::would_merge_with, each case in its own
//! throwaway repository:
//! squash_cases (a squash over a newer origin, a conflict, a zone at origin), resquash (the change ID kept on a
//! re-squash and new once origin took the squash) and would_merge_cases (Yes, and No with the conflicted files);
//! read_squash_cases reads a squash back with land::read_squash and its refusals; message_cases squashes with
//! land::squash_with_message (the caller's text before the request lines, Same on a repeat, a new message on an
//! unchanged zone a new commit with the same change ID); hook_cases checks hook refusals, message rewrites,
//! unreadable messages, repeats, and no hooks for Nothing or Conflicted; would_merge_with_cases checks ordered clean
//! and conflicting squashes without writes.

#[path = "squash_zit/fixture.rs"]
mod fixture;
#[path = "squash_zit/hook_cases.rs"]
mod hook_cases;
#[path = "squash_zit/message_cases.rs"]
mod message_cases;
#[path = "squash_zit/read_squash_cases.rs"]
mod read_squash_cases;
#[path = "squash_zit/resquash.rs"]
mod resquash;
#[path = "squash_zit/squash_cases.rs"]
mod squash_cases;
#[path = "squash_zit/would_merge_cases.rs"]
mod would_merge_cases;
#[path = "squash_zit/would_merge_with_cases.rs"]
mod would_merge_with_cases;
