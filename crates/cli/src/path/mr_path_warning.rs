//! mr_path_warning: the line `mr guard install` adds when the first `mr` on PATH is not this Mortar (S-133 (1)): the
//! guard hooks run `mortar`, but a person typing `mr` may reach another program (Homebrew's myrepos). Pure.

use std::path::Path;

/// None when `first` (the first `mr` on PATH) is Some and `same` (it is this build). Else one line naming that `mr`,
/// or saying there is none, and naming `mortar` at `this` (this binary's path) as the way in.
pub fn mr_path_warning(first: Option<&Path>, this: &Path, same: bool) -> Option<String> {
    let way_in = format!("run Mortar as mortar ({})", this.display());
    match first {
        Some(_) if same => None,
        Some(first) => Some(format!("the first mr on PATH is {}, not this Mortar: {way_in}", first.display())),
        None => Some(format!("no mr is on PATH: {way_in}")),
    }
}
