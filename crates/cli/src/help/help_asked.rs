//! help_asked: whether a command line asks for help, and for which topic.

use super::{find_help_entry, scan_help_words, HelpRequest, HelpTopic};

/// The line's globals are read by scan_help_words; W is its other words. Help is answered before parse_args reads
/// the line, so a line parse_args would refuse for its globals (a second `--json`, say) still gets help.
/// `mr help` (W is "help" alone) asks for the program help, `mr help <verb>` for that verb's (Unknown when it is
/// no verb), and `mr help` with more words is TooMany. Otherwise, with `--help` or `-h` seen: no W is the program
/// help, and a first word that is a verb asks for that verb's help (the rest are its arguments). Anything else is
/// None, so parse_args reads the line (an unknown verb with `--help` is its usage error).
pub fn help_asked(argv: &[String]) -> Option<HelpRequest> {
    let scan = scan_help_words(argv);
    let topic = match (scan.words.as_slice(), scan.flagged) {
        (["help"], _) => HelpTopic::Program,
        (["help", verb], _) => find_help_entry(verb)
            .map_or_else(|| HelpTopic::Unknown((*verb).to_string()), |(group, entry)| HelpTopic::Verb(group, entry)),
        (["help", ..], _) => HelpTopic::TooMany,
        ([], true) => HelpTopic::Program,
        ([verb, ..], true) => find_help_entry(verb).map(|(group, entry)| HelpTopic::Verb(group, entry))?,
        _ => return None,
    };
    Some(HelpRequest { json: scan.json, topic })
}
