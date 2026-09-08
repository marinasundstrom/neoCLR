use chrono::Utc;
use neoclr::{Limits, LoadedProgram, Value, assembler::parse_function_ref};

#[test]
fn snapshot_matches_host_instant_and_sample_survives_artifact_roundtrip() {
    let module =
        neoclr::frontend::compile(include_str!("../examples/source/local-clock.neo")).unwrap();
    let p = LoadedProgram::new(&neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap())
        .unwrap();
    p.verify().unwrap();
    let services = p
        .analyze_reachability(
            &[parse_function_ref("neoCLR.Runtime.LocalClock()").unwrap()],
            8,
        )
        .unwrap()
        .required_services();
    assert_eq!(services.len(), 2);
    assert!(services.contains(&neoclr::RuntimeService::LocalClock));
    assert!(services.contains(&neoclr::RuntimeService::ManagedArrays));
    let clock = p
        .resolve_function(&parse_function_ref("System.Clock::GetLocalNow()").unwrap())
        .unwrap();
    let before = Utc::now().timestamp_nanos_opt().unwrap();
    let value = clock.invoke(vec![], Limits::default()).unwrap().value;
    let after = Utc::now().timestamp_nanos_opt().unwrap();
    let Value::Object { fields, .. } = value else {
        panic!("snapshot expected")
    };
    let Value::Object { fields: date, .. } = &fields[0] else {
        panic!("date expected")
    };
    let Value::Object { fields: time, .. } = &fields[1] else {
        panic!("time expected")
    };
    let Value::Int32(days) = date[0] else {
        panic!("day number expected")
    };
    let Value::Int64(ticks) = time[0] else {
        panic!("ticks expected")
    };
    let Value::Int32(offset) = fields[2] else {
        panic!("offset expected")
    };
    let nanos =
        ((i64::from(days) - 719_162) * 86400 - i64::from(offset)) * 1_000_000_000 + ticks * 100;
    assert!(nanos >= before - 99 && nanos <= after);
    let execution = p.run(Limits::default()).unwrap();
    assert_eq!(execution.value, Value::Int32(0));
    assert_eq!(execution.output.len(), 10);
}

#[cfg(unix)]
#[test]
fn cli_respects_host_timezone_configuration() {
    for (zone, offset) in [("UTC", "0"), ("UTC-02", "7200")] {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_neoclr"))
            .args(["run", "examples/source/local-clock.neo"])
            .env("TZ", zone)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(
            stdout.contains(&format!("UTC offset in seconds\n{offset}\n")),
            "{stdout}"
        );
    }
}
