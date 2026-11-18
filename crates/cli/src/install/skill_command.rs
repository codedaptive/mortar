//! skill_command: The name the skill text uses for Mortar: "mr" or "mortar" (spec 07 item 6, S-177).
//! Pure: the caller gives where `~/.local/bin` and the first `mr` stand on PATH.

/// "mr" exactly when, once install is done, the first `mr` on PATH is Mortar; else "mortar". `link_present`: Mortar's
/// link stands at `~/.local/bin/mr` once install is done. `bin_at`: the PATH entry index of `~/.local/bin`, None when
/// it is not on PATH. `first_mr`: the PATH entry index of the first `mr` before install, and whether it is Mortar.
/// The link is first when it is present, `~/.local/bin` is on PATH, and no `mr` stands in an earlier entry (an `mr`
/// found at `bin_at` itself is the file at the link's path, which the link now is). Otherwise the first `mr` is the
/// one found before install.
pub fn skill_command(link_present: bool, bin_at: Option<usize>, first_mr: Option<(usize, bool)>) -> &'static str {
    let link_first = link_present && bin_at.is_some_and(|bin| first_mr.is_none_or(|(at, _)| at >= bin));
    if link_first || first_mr.is_some_and(|(_, mortar)| mortar) {
        "mr"
    } else {
        "mortar"
    }
}
