//! The calendar the notebook's dates are read against.
//!
//! The Core carries no dependencies, so the calendar is its own: a date is
//! `YYYY-MM-DD` and a timestamp is RFC 3339, both proleptic Gregorian. The
//! day number turns either into an integer, so an age is a subtraction and
//! a deadline a comparison — and the Core still holds no clock, since today
//! is a value the host hands in.

pub(crate) fn is_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    let shaped = bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_digit() || *byte == b'-');
    if !shaped {
        return false;
    }
    let Ok(year) = value[0..4].parse::<u16>() else {
        return false;
    };
    let Ok(month) = value[5..7].parse::<u8>() else {
        return false;
    };
    let Ok(day) = value[8..10].parse::<u8>() else {
        return false;
    };
    (1..=12).contains(&month) && (1..=days_in_month(year, month)).contains(&day)
}

/// The date's civil day number (days since 1970-01-01, Howard Hinnant's
/// days-from-civil), taking the date part of a timestamp; `None` when the
/// value is not a valid date. Day arithmetic on notebook dates is a
/// subtraction of two of these — a host derives ages the same way the
/// Status clocks do.
#[must_use]
pub fn day_number(value: &str) -> Option<i64> {
    let date = match value.split_once(['T', 't']) {
        Some(_) if !is_timestamp(value) => return None,
        Some((date, _)) => date,
        None => value,
    };
    if !is_date(date) {
        return None;
    }
    let year: i64 = date[0..4].parse().ok()?;
    let month: i64 = date[5..7].parse().ok()?;
    let day: i64 = date[8..10].parse().ok()?;
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let day_of_year = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    Some(era * 146_097 + day_of_era - 719_468)
}

fn days_in_month(year: u16, month: u8) -> u8 {
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    match month {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// RFC 3339: `<date>T<HH:MM:SS>[.fraction](Z|±HH:MM)`, with the
/// lower-case `t` and `z` the grammar's case-insensitive literals allow.
pub(crate) fn is_timestamp(value: &str) -> bool {
    let Some((date, time)) = value.split_once(['T', 't']) else {
        return false;
    };
    if !is_date(date) {
        return false;
    }
    let Some(offset_start) = time.find(['Z', 'z', '+', '-']) else {
        return false;
    };
    let (clock, offset) = time.split_at(offset_start);
    let clock_ok = match clock.split_once('.') {
        Some((base, fraction)) => {
            is_clock(base) && !fraction.is_empty() && fraction.bytes().all(|b| b.is_ascii_digit())
        }
        None => is_clock(clock),
    };
    let offset_ok = offset.eq_ignore_ascii_case("Z")
        || offset
            .strip_prefix(['+', '-'])
            .and_then(|hhmm| hhmm.split_once(':'))
            .is_some_and(|(hours, minutes)| {
                hours.len() == 2
                    && minutes.len() == 2
                    && hours.parse::<u8>().is_ok_and(|h| h <= 23)
                    && minutes.parse::<u8>().is_ok_and(|m| m <= 59)
            });
    clock_ok && offset_ok
}

fn is_clock(clock: &str) -> bool {
    let bytes = clock.as_bytes();
    bytes.len() == 8
        && bytes[2] == b':'
        && bytes[5] == b':'
        && clock[0..2].parse::<u8>().is_ok_and(|hours| hours <= 23)
        && clock[3..5].parse::<u8>().is_ok_and(|minutes| minutes <= 59)
        && clock[6..8].parse::<u8>().is_ok_and(|seconds| seconds <= 59)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The `T` and the `Z` are case-insensitive literals, so the two
    /// spellings are the same instant. A value the grammar accepts and the
    /// calendar cannot read would fall out of every clock at once, aged as
    /// today and never due.
    #[test]
    fn a_timestamp_reads_the_same_day_however_its_separator_is_cased() {
        assert_eq!(
            day_number("2026-08-24t12:34:56z"),
            day_number("2026-08-24T12:34:56Z")
        );
        assert_eq!(day_number("2026-08-24t12:34:56z"), day_number("2026-08-24"));
        assert!(is_timestamp("2026-08-24t12:34:56z"));
    }

    /// A value shaped like a timestamp and malformed inside it is no date:
    /// taking the part before the separator would read a broken value as a
    /// sound one.
    #[test]
    fn a_malformed_timestamp_is_no_day_at_all() {
        for value in ["2026-08-24t99:00:00z", "2026-08-24T12:34:56", "2026-08-24t"] {
            assert_eq!(day_number(value), None, "{value}");
        }
    }
}
