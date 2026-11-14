//! Zone integration tests of land and back_out (SPEC-land 03-land.md §Tests): landing (cases 1 to 4), backing_out
//! (case 5) and kills (cases 6 and 7, with their worker), each case in its own throwaway repository; and
//! state_display, LandError's State text, which needs no repository.

#[path = "landing_zit/backing_out.rs"]
mod backing_out;
#[path = "landing_zit/fixture.rs"]
mod fixture;
#[path = "landing_zit/kills.rs"]
mod kills;
#[path = "landing_zit/landing.rs"]
mod landing;
#[path = "landing_zit/state_display.rs"]
mod state_display;
