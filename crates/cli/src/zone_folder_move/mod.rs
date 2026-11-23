//! The zone-folder move: a zone folder brought from the commit it stands on to its zone branch's tip, its outcome in
//! the cli's words and the next parts that clear it; shared by `mr after-publish` and `mr folder refresh`
//! (`mr commit-files` uses find_zone_old and zone_values).

mod bring_zone_folder;
mod find_zone_old;
mod folder_part;
mod folder_parts;
mod folder_refusal;
mod zone_folder_lines;
mod zone_folder_outcome;
mod zone_old;
mod zone_values;

#[cfg(test)]
mod find_zone_old_tests;
#[cfg(test)]
mod folder_part_tests;
#[cfg(test)]
mod zone_folder_lines_tests;
#[cfg(test)]
mod zone_values_tests;

pub(crate) use bring_zone_folder::bring_zone_folder;
pub(crate) use find_zone_old::find_zone_old;
pub(crate) use folder_part::folder_part;
pub(crate) use folder_parts::FOLDER_PARTS;
pub(crate) use folder_refusal::FolderRefusal;
pub(crate) use zone_folder_lines::zone_folder_lines;
pub(crate) use zone_folder_outcome::ZoneFolderOutcome;
pub(crate) use zone_old::ZoneOld;
pub(crate) use zone_values::zone_values;
