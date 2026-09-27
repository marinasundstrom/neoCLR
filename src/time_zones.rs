//! Pinned IANA rules. Civil policy and public errors remain in the Raven library.
use crate::{Value, metadata::Type};
use chrono::{DateTime, Datelike, LocalResult, Offset, TimeZone, Utc};
use chrono_tz::Tz;

const TICKS_PER_SECOND: i64 = 10_000_000;
fn timestamp(ticks: i64) -> Option<DateTime<Utc>> {
    let value = DateTime::from_timestamp(
        ticks.div_euclid(TICKS_PER_SECOND),
        (ticks.rem_euclid(TICKS_PER_SECOND) * 100) as u32,
    )?;
    (1900..=2099).contains(&value.year()).then_some(value)
}
fn ticks(value: DateTime<Tz>) -> i64 {
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
pub(crate) fn map_local(id: &str, local_ticks: i64) -> Value {
    // Status: -1 out of range/unknown; 0 gap; 1 unique; 2 ordered overlap.
    let values = match id.parse::<Tz>().ok().zip(timestamp(local_ticks)) {
        None => vec![-1],
        Some((zone, local)) => match zone.from_local_datetime(&local.naive_utc()) {
            LocalResult::None => vec![0],
            LocalResult::Single(value) => {
                let instant = ticks(value);
                if offset(id, instant) == i32::MIN {
                    vec![-1]
                } else {
                    vec![1, instant]
                }
            }
            LocalResult::Ambiguous(first, second) => {
                let (first, second) = (ticks(first), ticks(second));
                if offset(id, first) == i32::MIN || offset(id, second) == i32::MIN {
                    vec![-1]
                } else {
                    vec![2, first.min(second), first.max(second)]
                }
            }
        },
    };
    Value::Array {
        element: Type::Int64,
        elements: values.into_iter().map(Value::Int64).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn at(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> i64 {
        Utc.with_ymd_and_hms(year, month, day, hour, minute, 0)
            .single()
            .unwrap()
            .timestamp()
            * TICKS_PER_SECOND
    }
    fn mapped(id: &str, value: i64) -> Vec<i64> {
        let Value::Array { elements, .. } = map_local(id, value) else {
            panic!("Expected array")
        };
        elements
            .into_iter()
            .map(|v| {
                if let Value::Int64(n) = v {
                    n
                } else {
                    panic!("Expected tick value")
                }
            })
            .collect()
    }
    #[test]
    fn transitions_preserve_zero_one_and_two_instant_mappings() {
        assert_eq!(mapped("Europe/Stockholm", at(2024, 3, 31, 2, 30)), [0]);
        assert_eq!(
            mapped("Europe/Stockholm", at(2024, 10, 27, 2, 30)),
            [2, at(2024, 10, 27, 0, 30), at(2024, 10, 27, 1, 30)]
        );
        assert_eq!(
            mapped("America/New_York", at(2024, 11, 3, 1, 30)),
            [2, at(2024, 11, 3, 5, 30), at(2024, 11, 3, 6, 30)]
        );
        assert_eq!(
            mapped("Australia/Lord_Howe", at(2024, 4, 7, 1, 45)),
            [2, at(2024, 4, 6, 14, 45), at(2024, 4, 6, 15, 15)]
        );
        assert_eq!(mapped("Pacific/Apia", at(2011, 12, 30, 12, 0)), [0]);
        assert_eq!(
            mapped("Asia/Kolkata", at(2024, 1, 1, 12, 0)),
            [1, at(2024, 1, 1, 6, 30)]
        );
    }
    #[test]
    fn exact_ticks_historical_seconds_and_bounds() {
        assert_eq!(offset("Europe/Paris", at(1900, 1, 2, 0, 0)), 561);
        assert_eq!(mapped("UTC", -1), [1, -1]);
        let value = at(2024, 10, 27, 2, 30) + 1234567;
        let results = mapped("Europe/Stockholm", value);
        for instant in &results[1..] {
            assert_eq!(
                *instant + i64::from(offset("Europe/Stockholm", *instant)) * TICKS_PER_SECOND,
                value
            );
        }
        for value in [
            i64::MIN,
            i64::MAX,
            at(1899, 12, 31, 0, 0),
            at(2100, 1, 1, 0, 0),
        ] {
            assert_eq!(mapped("UTC", value), [-1]);
            assert_eq!(offset("UTC", value), i32::MIN);
        }
        assert!(exists("Europe/Stockholm"));
        assert!(!exists("europe/stockholm"));
        assert!(!exists("Not/AZone"));
        assert_eq!(mapped("Not/AZone", 0), [-1]);
        assert_eq!(chrono_tz::IANA_TZDB_VERSION, "2025b");
    }
}
