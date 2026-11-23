//! The words of `mr folder` after its globals, read into the sub-verb they ask for.

use std::path::PathBuf;

use super::FolderCommand;

/// The first word is the sub-verb: zone, request, remove, refresh, attempts, keep or drop; each takes exactly its
/// positional words. `--start <commit>` or `--start=<commit>` is zone's only flag, at most once. `--paths` is
/// required for request, once, after its three positional words, and takes the words after it up to the next --with
/// (at least one). `--with <zone>` or `--with=<zone>`, request's partner, is repeatable anywhere after those three
/// words, kept in order; a partner given twice or equal to the request's zone is refused. n and k are whole numbers
/// from 1 to u32::MAX written without sign or leading zero. An empty word, any other dashed word (a path, a --start
/// or --with value starting with "-" included) or a word too many is refused; the Err is the reason and names it.
pub fn parse_folder_args(rest: &[String]) -> Result<FolderCommand, String> {
    let Some((sub, words)) = rest.split_first() else {
        return Err("folder needs a sub-verb: zone, request, remove, refresh, attempts, keep or drop".to_string());
    };
    match sub.as_str() {
        "zone" => zone(words),
        "request" => request(words),
        "remove" => {
            let [path] = positional("remove", ["<path>"], words)?;
            Ok(FolderCommand::Remove { path: PathBuf::from(path) })
        }
        "refresh" => {
            let [path] = positional("refresh", ["<path>"], words)?;
            Ok(FolderCommand::Refresh { path: PathBuf::from(path) })
        }
        "attempts" => {
            let [n, dir] = positional("attempts", ["<n>", "<dir>"], words)?;
            Ok(FolderCommand::Attempts { n: count("<n>", n)?, dir: PathBuf::from(dir) })
        }
        "keep" => {
            let [k] = positional("keep", ["<k>"], words)?;
            Ok(FolderCommand::Keep { k: count("<k>", k)? })
        }
        "drop" => {
            let [] = positional("drop", [], words)?;
            Ok(FolderCommand::Drop)
        }
        other => Err(format!("unknown folder sub-verb {other:?}")),
    }
}

/// `folder zone <zone> <path> [--start <commit>]`.
fn zone(words: &[String]) -> Result<FolderCommand, String> {
    let (mut start, mut rest) = (None, Vec::new());
    let mut words = words.iter();
    while let Some(word) = words.next() {
        let value = match (word.as_str(), word.strip_prefix("--start=")) {
            ("--start", _) => words.next().map(String::as_str),
            (_, Some(value)) => Some(value),
            _ => {
                rest.push(word.clone());
                continue;
            }
        };
        match value {
            None | Some("") => return Err("--start needs a commit".to_string()),
            Some(value) if value.starts_with('-') => return Err(format!("--start needs a commit, not {value:?}")),
            Some(_) if start.is_some() => return Err("--start is given twice".to_string()),
            Some(value) => start = Some(value.to_string()),
        }
    }
    let [zone, path] = positional("zone", ["<zone>", "<path>"], &rest)?;
    Ok(FolderCommand::Zone { zone: zone.to_string(), path: PathBuf::from(path), start })
}

/// `folder request <zone> <request> <path> --paths <path>... [--with <zone>]...`; a word between the three
/// positional ones and --paths that is no --with goes to positional, which refuses it.
fn request(words: &[String]) -> Result<FolderCommand, String> {
    let Some(at) = words.iter().position(|word| word == "--paths") else {
        return Err("folder request needs --paths <path>...".to_string());
    };
    let (head, tail) = (&words[..at], &words[at + 1..]);
    let (mut partners, mut named) = (Vec::new(), head[..head.len().min(3)].to_vec());
    named.extend(take_partners(&head[named.len()..], &mut partners)?);
    let [zone, request, path] = positional("request", ["<zone>", "<request>", "<path>"], &named)?;
    let paths_end = tail.iter().position(|word| word == "--with" || word.starts_with("--with=")).unwrap_or(tail.len());
    let paths = &tail[..paths_end];
    if paths.is_empty() {
        return Err("--paths needs at least one path".to_string());
    }
    if tail.iter().any(|word| word == "--paths") {
        return Err("--paths is given twice".to_string());
    }
    if let Some(word) = paths.iter().find(|word| word.is_empty() || word.starts_with('-')) {
        return Err(format!("unknown folder request argument {word:?}"));
    }
    if let Some(extra) = take_partners(&tail[paths_end..], &mut partners)?.first() {
        return Err(format!("unexpected word {extra:?} in folder request"));
    }
    check_partners(zone, &partners)?;
    let (zone, request, path, paths) = (zone.to_string(), request.to_string(), PathBuf::from(path), paths.to_vec());
    Ok(FolderCommand::Request { zone, request, path, paths, partners })
}

/// Moves each --with's zone from `words` onto `partners`, in order, and answers the other words; a --with with no
/// word, an empty one or a dashed one is refused.
fn take_partners(words: &[String], partners: &mut Vec<String>) -> Result<Vec<String>, String> {
    let mut rest = Vec::new();
    let mut words = words.iter();
    while let Some(word) = words.next() {
        let value = match (word.as_str(), word.strip_prefix("--with=")) {
            ("--with", _) => words.next().map(String::as_str),
            (_, Some(value)) => Some(value),
            _ => {
                rest.push(word.clone());
                continue;
            }
        };
        match value {
            None | Some("") => return Err("--with needs a zone".to_string()),
            Some(value) if value.starts_with('-') => return Err(format!("--with needs a zone, not {value:?}")),
            Some(value) => partners.push(value.to_string()),
        }
    }
    Ok(rest)
}

/// Refuses a partner that is `zone` itself or one given twice, the first such in the order given.
fn check_partners(zone: &str, partners: &[String]) -> Result<(), String> {
    match partners.iter().enumerate().find(|(at, partner)| *partner == zone || partners[..*at].contains(partner)) {
        Some((_, partner)) if partner == zone => Err(format!("zone {zone} cannot be its own partner")),
        Some((_, partner)) => Err(format!("partner {partner} is given twice")),
        None => Ok(()),
    }
}

/// `sub`'s positional words, exactly as many as `names`; an empty or dashed word, a word too many or a missing one
/// is refused.
fn positional<'a, const N: usize>(sub: &str, names: [&str; N], words: &'a [String]) -> Result<[&'a str; N], String> {
    if let Some(word) = words.iter().find(|word| word.is_empty() || word.starts_with('-')) {
        return Err(format!("unknown folder {sub} argument {word:?}"));
    }
    if let Some(extra) = words.get(N) {
        return Err(format!("unexpected word {extra:?} in folder {sub}"));
    }
    let found: Vec<&str> = words.iter().map(String::as_str).collect();
    found.try_into().map_err(|_| format!("folder {sub} needs {}", names.join(" ")))
}

/// A whole number from 1 to u32::MAX, digits only, no leading zero.
fn count(name: &str, word: &str) -> Result<u32, String> {
    let canonical = word.bytes().all(|byte| byte.is_ascii_digit()) && !word.starts_with('0');
    canonical
        .then(|| word.parse::<u32>().ok())
        .flatten()
        .ok_or_else(|| format!("{name} is a whole number from 1 to {}, not {word:?}", u32::MAX))
}
