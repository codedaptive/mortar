//! A unix time as an RFC 3339 UTC timestamp.

/// "YYYY-MM-DDTHH:MM:SSZ" for `unix_secs` seconds after 1970-01-01T00:00:00Z, in the proleptic Gregorian calendar.
pub fn rfc3339_utc(unix_secs: u64) -> String {
    let (year, month, day) = civil_from_days(unix_secs / 86_400);
    let secs = unix_secs % 86_400;
    format!("{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z", secs / 3600, secs / 60 % 60, secs % 60)
}

/// Howard Hinnant's civil_from_days for days since 1970-01-01: it counts 400-year eras from 0000-03-01, so the leap
/// day falls at the end of each computed year and needs no special case.
fn civil_from_days(days: u64) -> (u64, u64, u64) {
    let shifted = days + 719_468;
    let era = shifted / 146_097;
    let day_of_era = shifted % 146_097;
    let year_of_era = (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_from_march = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_from_march + 2) / 5 + 1;
    let month = if month_from_march < 10 { month_from_march + 3 } else { month_from_march - 9 };
    (era * 400 + year_of_era + u64::from(month <= 2), month, day)
}
