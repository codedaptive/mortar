//! decode_request_base: reads the text encode_request_base writes back into a RequestBase.
//! decode_request_base_tests covers hand-written texts, a round trip through encode_request_base and each refusal.
//! Why strict: only encode_request_base writes these values, so a text with other members, or members in another
//! order, is not a base join recorded, and the cli must not put a folder record's base back from it.

use crate::partner::Partner;
use crate::request_base::RequestBase;
use gitio::Oid;
use oplog::{parse_json, JsonError, JsonValue};

/// One JSON object with exactly the members base then partners: base an object id as a string, partners an array of
/// objects with exactly the members zone (a string) then tip (an object id as a string). Whitespace between tokens
/// is allowed, as parse_json reads it. Anything else is refused with the reason, naming the member or partner.
pub(crate) fn decode_request_base(text: &str) -> Result<RequestBase, String> {
    let value = parse_json(text).map_err(|error| match error {
        JsonError::EndOfInput => "the text ends before the base is complete".to_string(),
        JsonError::Invalid(why) => format!("the text is not JSON: {why}"),
    })?;
    let [base, partners] = members(value, "the base", ["base", "partners"])?;
    let base = id(base, "base")?;
    let JsonValue::Array(partners) = partners else {
        return Err("partners is not an array".to_string());
    };
    let partners = partners.into_iter().enumerate().map(|(i, item)| partner(item, i)).collect::<Result<_, _>>()?;
    Ok(RequestBase { base, partners })
}

/// Partner `i` of the array: exactly `{"zone":<string>,"tip":"<hex>"}`.
fn partner(value: JsonValue, i: usize) -> Result<Partner, String> {
    let [zone, tip] = members(value, &format!("partner {i}"), ["zone", "tip"])?;
    let JsonValue::String(zone) = zone else {
        return Err(format!("partner {i}'s zone is not a string"));
    };
    Ok(Partner { zone, tip: id(tip, &format!("partner {i}'s tip"))? })
}

/// The values of an object with exactly the two members `names`, in that order; `what` names the object.
fn members(value: JsonValue, what: &str, names: [&str; 2]) -> Result<[JsonValue; 2], String> {
    let refusal = || format!("{what} is not an object with exactly the members {} then {}", names[0], names[1]);
    let JsonValue::Object(pairs) = value else {
        return Err(refusal());
    };
    match <[(String, JsonValue); 2]>::try_from(pairs) {
        Ok([(first, a), (second, b)]) if first == names[0] && second == names[1] => Ok([a, b]),
        _ => Err(refusal()),
    }
}

/// A string holding an object id; `what` names the member.
fn id(value: JsonValue, what: &str) -> Result<Oid, String> {
    let JsonValue::String(hex) = value else {
        return Err(format!("{what} is not a string"));
    };
    Oid::parse(&hex).map_err(|_| format!("{what} {hex:?} is not a git object id"))
}
