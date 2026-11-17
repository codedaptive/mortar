//! The words of `mortar after-publish` after its globals: the published zone and each --move.

use super::AfterPublishArgs;
use crate::config::Config;

/// Reads `rest`; `dash_dash` is where the line's `--` stood, if it had one. The first failure, in this order, is
/// the reason returned: any `--`; then the words left to right (a --move with no word after it, any other dashed
/// word, a second plain word); then no published zone; then each zone in order, the published one first and each
/// --move after it: a zone the config does not name (with the config's zones in name order), or one named twice.
pub fn parse_after_publish_args(
    rest: &[String],
    dash_dash: Option<usize>,
    config: &Config,
) -> Result<AfterPublishArgs, String> {
    if dash_dash.is_some() {
        return Err("after-publish takes no `--`".to_string());
    }
    let mut published: Option<String> = None;
    let mut moves = Vec::new();
    let mut words = rest.iter();
    while let Some(word) = words.next() {
        match word.as_str() {
            "--move" => match words.next() {
                Some(zone) => moves.push(zone.clone()),
                None => return Err("--move needs a zone".to_string()),
            },
            dashed if dashed.starts_with('-') => return Err(format!("unexpected word {dashed:?}")),
            plain if published.is_some() => return Err(format!("unexpected word {plain:?} after the zone")),
            plain => published = Some(plain.to_string()),
        }
    }
    let Some(published) = published else {
        return Err("after-publish needs the zone whose squash was published".to_string());
    };
    let mut seen: Vec<&str> = Vec::new();
    for zone in std::iter::once(&published).chain(&moves) {
        if !config.zones.contains_key(zone) {
            let names = config.zones.keys().map(String::as_str).collect::<Vec<_>>().join(", ");
            return Err(format!("unknown zone {zone:?}: the config names {names}"));
        }
        if seen.contains(&zone.as_str()) {
            return Err(format!("zone {zone:?} is named twice"));
        }
        seen.push(zone);
    }
    Ok(AfterPublishArgs { published, moves })
}
