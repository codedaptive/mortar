//! A resolve's report when the resolve left the request on the zone tip without the partners its record named.

use crate::report::{OutValue, Report};

/// `report` with a line saying the request left its set's merge and stands on the zone tip without `partners`
/// (named in order, ", " between them), so its unit tests run without them and the set's tests run it with them
/// after mr join (S-233); a `left_partners` field; and next "mr join". Everything else as given.
pub(crate) fn left_set_report(report: Report, partners: &[String]) -> Report {
    let mut lines = report.lines;
    lines.push(format!(
        "the request has left its set's merge: it stands on the zone tip without partners {}; its unit tests run \
         without them, and the set's tests run it with them after mr join",
        partners.join(", ")
    ));
    let mut fields = report.fields;
    fields.push(("left_partners".to_string(), OutValue::Texts(partners.to_vec())));
    Report { lines, fields, next: "mr join".to_string(), ..report }
}
