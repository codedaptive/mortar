//! Zone integration tests of land and back_out: landing (a fast-forward, a merge, a conflict, a second landing while
//! one is in flight), backing_out (back_out after a merge landing) and kills (a merge landing killed before or after
//! its update-ref, with their worker), each case in its own throwaway repository; and state_display, LandError's
//! State text, which needs no repository.

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
