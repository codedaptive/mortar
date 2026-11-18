//! plan_install: Install's whole plan: placement, the link, the skills, every refusal before any step (spec 07 item 7).
//! Pure: it decides from the facts install read. Placement is planned first and its refusal ends the plan there;
//! then the link, whose refusal ends it; then the skills, whose text names the command the link decided.

use super::{
    plan_link, plan_placement, plan_skills, skill_command, skill_text, HarnessSpec, InstallFacts, InstallItem,
    InstallOptions, InstallPaths, InstallPlan,
};
use crate::report::Report;

/// The plan, or the first refusal: placement's, then the link's, then the skills'. Steps and items in that order;
/// when the binary is placed and `~/.local/bin` is not on PATH, a "not on PATH" item follows them. The next step is
/// the link's when it forces one; else, placed and `~/.local/bin` not on PATH, to add it to PATH; else
/// "<command> init" in the repository Mortar is to keep. `command` is skill_command's choice, carried in the plan.
pub fn plan_install(
    facts: &InstallFacts,
    options: &InstallOptions,
    paths: &InstallPaths,
    skip: Option<String>,
    chosen: &[&'static HarnessSpec],
    wiring_note: Option<String>,
) -> Result<InstallPlan, Report> {
    let placed = skip.is_none();
    let (place_step, place_item) = plan_placement(facts, paths, skip.as_deref())?;
    let link = plan_link(facts, paths, placed, options.short_name)?;
    let first_mr = facts.first_mr_at.zip(facts.first_mr.as_ref().map(|(_, mortar)| *mortar));
    let command = skill_command(link.present, facts.bin_at, first_mr);
    let (skill_steps, skill_items) = plan_skills(facts, chosen, &skill_text(command), wiring_note)?;
    let off_path = placed && facts.bin_at.is_none();
    let steps = place_step.into_iter().chain(link.step).chain(skill_steps).collect();
    let mut items = vec![place_item, link.item];
    items.extend(skill_items);
    if off_path {
        items.push(InstallItem {
            action: "not on PATH".to_string(),
            path: paths.bin_dir.clone(),
            note: "mortar and mr cannot be found by name, and the guard hooks run mortar".to_string(),
        });
    }
    let next = match (link.next, off_path) {
        (Some(next), _) => next,
        (None, true) => format!("add {} to PATH", paths.bin_dir.display()),
        (None, false) => format!("{command} init, in the repository Mortar is to keep"),
    };
    Ok(InstallPlan { steps, items, next, command })
}
