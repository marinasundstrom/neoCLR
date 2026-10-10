//! Shared allocation-free pinned IANA lookup and offset arithmetic.
use chrono::{DateTime, Datelike, Offset, Utc};
use chrono_tz::Tz;

const TICKS_PER_SECOND: i64 = 10_000_000;
pub(crate) fn timestamp(ticks: i64) -> Option<DateTime<Utc>> {
    let value = DateTime::from_timestamp(
        ticks.div_euclid(TICKS_PER_SECOND),
        (ticks.rem_euclid(TICKS_PER_SECOND) * 100) as u32,
    )?;
    (1900..=2099).contains(&value.year()).then_some(value)
}
pub(crate) fn ticks(value: DateTime<Tz>) -> i64 {
    value.timestamp() * TICKS_PER_SECOND + i64::from(value.timestamp_subsec_nanos() / 100)
}
pub(crate) fn exists(id: &str) -> bool {
    id.parse::<Tz>().is_ok()
}
pub(crate) fn offset(id: &str, instant: i64) -> i32 {
    let Some((zone, instant)) = id.parse::<Tz>().ok().zip(timestamp(instant)) else {
        return i32::MIN;
    };
    let local = instant.with_timezone(&zone);
    if !(1900..=2099).contains(&local.year()) {
        return i32::MIN;
    }
    local.offset().fix().local_minus_utc()
}
