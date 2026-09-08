//! One host-local clock reading, projected into ordinary library values.
use crate::{Fault, Value, metadata::Type};
use chrono::{DateTime, Datelike, Local, Offset, TimeZone, Timelike};

pub(crate) fn read_local() -> Result<Value, Fault> {
    components(Local::now())
}

fn components<T: TimeZone>(now: DateTime<T>) -> Result<Value, Fault> {
    if !(1..=9999).contains(&now.year()) || now.nanosecond() >= 1_000_000_000 {
        return Err(Fault::new(
            "system local time is outside the supported Date/Time range",
        ));
    }
    let values = [
        now.year(),
        now.month() as i32,
        now.day() as i32,
        now.hour() as i32,
        now.minute() as i32,
        now.second() as i32,
        (now.nanosecond() / 100) as i32,
        now.offset().fix().local_minus_utc(),
    ];
    Ok(Value::Array {
        element: Type::Int32,
        elements: values.into_iter().map(Value::Int32).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{FixedOffset, Utc};
    #[test]
    fn one_instant_keeps_date_time_and_offset_consistent_across_midnight() {
        let utc = Utc
            .with_ymd_and_hms(2024, 2, 29, 23, 30, 45)
            .unwrap()
            .with_nanosecond(123_456_789)
            .unwrap();
        for (offset, expected) in [
            (7200, [2024, 3, 1, 1, 30, 45, 1_234_567, 7200]),
            (-3600, [2024, 2, 29, 22, 30, 45, 1_234_567, -3600]),
        ] {
            let local = utc.with_timezone(&FixedOffset::east_opt(offset).unwrap());
            assert_eq!(
                components(local).unwrap(),
                Value::Array {
                    element: Type::Int32,
                    elements: expected.into_iter().map(Value::Int32).collect()
                }
            );
        }
        assert!(components(Utc.with_ymd_and_hms(0, 1, 1, 0, 0, 0).unwrap()).is_err());
        assert!(components(Utc.with_ymd_and_hms(10000, 1, 1, 0, 0, 0).unwrap()).is_err());
    }
}
