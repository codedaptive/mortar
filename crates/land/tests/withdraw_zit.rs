//! Zone integration tests of land::withdraw, each case in its own throwaway repository: withdraw_cases (a withdraw
//! deletes the squash ref in one land-withdraw op and answers its commit and change ID; a repeat answers Absent and
//! writes nothing; a squash already in origin and the squash in the landing state are refused and move nothing; a
//! squash after a withdraw draws a new change ID).

#[path = "withdraw_zit/fixture.rs"]
mod fixture;
#[path = "withdraw_zit/withdraw_cases.rs"]
mod withdraw_cases;
