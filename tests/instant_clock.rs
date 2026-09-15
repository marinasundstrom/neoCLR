use chrono::{Datelike, Local, TimeZone, Timelike, Utc};
use neoclr::{Limits, LoadedProgram, Value, assemble, assembler::parse_function_ref};

fn program() -> LoadedProgram {
    let app = assemble(
        r#"
.module InstantTest
.entry Now
.function Now() -> Int64
.local System.Instant instant
newobj instance System.SystemClock::.ctor()
castclass System.Clock
callvirt instance System.Clock::get_Now()
stloc instant
ldloca instant
call instance System.Instant::get_UnixTimeTicks()
ret
.end
.function LocalAt(Int64 ticks) -> System.LocalDateTime
.local System.Instant instant
ldarg ticks
call System.Instant::FromUnixTimeTicks(Int64)
stloc instant
ldloca instant
call instance System.Instant::ToLocalDateTime()
ret
.end
"#,
    )
    .unwrap();
    LoadedProgram::with_library(&app, neoclr::library::system().unwrap()).unwrap()
}

#[test]
fn system_clock_dispatch_matches_host_and_reports_wall_clock_service() {
    let p = program();
    p.verify().unwrap();
    let services = p
        .analyze_reachability(
            &[parse_function_ref("neoCLR.Runtime.UnixTimeTicks()").unwrap()],
            8,
        )
        .unwrap()
        .required_services();
    assert_eq!(services, vec![neoclr::RuntimeService::WallClock]);
    let before = Utc::now().timestamp_nanos_opt().unwrap().div_euclid(100);
    let actual = p.run(Limits::default()).unwrap().value;
    let after = Utc::now().timestamp_nanos_opt().unwrap().div_euclid(100);
    let Value::Int64(ticks) = actual else {
        panic!("ticks expected")
    };
    assert!((before..=after).contains(&ticks));
}

#[test]
fn conversion_uses_supplied_instant_and_system_zone() {
    let p = program();
    let function = p
        .resolve_function(&parse_function_ref("LocalAt(Int64)").unwrap())
        .unwrap();
    for ticks in [-1_i64, 0, 17_092_476_451_234_567] {
        let expected = Utc
            .timestamp_opt(
                ticks.div_euclid(10_000_000),
                (ticks.rem_euclid(10_000_000) * 100) as u32,
            )
            .unwrap()
            .with_timezone(&Local);
        let Value::Object { fields, .. } = function
            .invoke(vec![Value::Int64(ticks)], Limits::default())
            .unwrap()
            .value
        else {
            panic!("local date time expected")
        };
        let Value::Object { fields: date, .. } = &fields[0] else {
            panic!("date expected")
        };
        let Value::Object { fields: time, .. } = &fields[1] else {
            panic!("time expected")
        };
        assert_eq!(date[0], Value::Int32(expected.num_days_from_ce() - 1));
        assert_eq!(
            time[0],
            Value::Int64(
                i64::from(expected.num_seconds_from_midnight()) * 10_000_000
                    + i64::from(expected.nanosecond() / 100)
            )
        );
    }
    assert!(
        function
            .invoke(vec![Value::Int64(i64::MAX)], Limits::default())
            .is_err()
    );
}
