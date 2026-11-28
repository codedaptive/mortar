//! Zone integration tests of land and back_out: landing (a fast-forward, a merge, a conflict, a second squash of the
//! same zone while the first is in flight), batch (zones appended to one batch, repeats, a duplicate zone, back_out
//! of the batch), backing_out (back_out after a merge landing) and kills (a merge landing killed before or after its
//! update-ref, or a batch's second member killed before its update-ref, with their worker), and hooks (a merge
//! landing runs no commit hooks), each case in its own throwaway repository.

#[path = "landing_zit/batch.rs"]
mod batch;
#[path = "landing_zit/backing_out.rs"]
mod backing_out;
#[path = "landing_zit/fixture.rs"]
mod fixture;
#[path = "landing_zit/hooks.rs"]
mod hooks;
#[path = "landing_zit/kills.rs"]
mod kills;
#[path = "landing_zit/landing.rs"]
mod landing;
