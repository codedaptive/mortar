//! Zone integration tests of land::squash and land::would_merge (SPEC-land 01-squash.md §Tests; case 5 and the new
//! change ID after origin took the squash are from its squash rules), each case in its own throwaway repository:
//! squash_cases (cases 1, 3, 5), resquash (case 2) and would_merge_cases (case 4); read_squash_cases reads a
//! squash back with land::read_squash and its refusals.

#[path = "squash_zit/fixture.rs"]
mod fixture;
#[path = "squash_zit/read_squash_cases.rs"]
mod read_squash_cases;
#[path = "squash_zit/resquash.rs"]
mod resquash;
#[path = "squash_zit/squash_cases.rs"]
mod squash_cases;
#[path = "squash_zit/would_merge_cases.rs"]
mod would_merge_cases;
