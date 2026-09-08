use neoclr::{Limits, LoadedProgram, Value, assembler::parse_function_ref};

fn program() -> LoadedProgram {
    let mut source = String::from(".module Dates\n");
    for (name, constructor, params, args) in [
        (
            "Date",
            "Create",
            "Int32 year, Int32 month, Int32 day",
            "ldarg year\nldarg month\nldarg day",
        ),
        ("Date", "FromDayNumber", "Int32 number", "ldarg number"),
        (
            "Time",
            "Create",
            "Int32 hour, Int32 minute, Int32 second, Int32 fractionTicks",
            "ldarg hour\nldarg minute\nldarg second\nldarg fractionTicks",
        ),
        ("Time", "FromTicks", "Int64 ticks", "ldarg ticks"),
    ] {
        let sig = params
            .split(',')
            .map(|p| p.trim().split(' ').next().unwrap())
            .collect::<Vec<_>>()
            .join(",");
        source.push_str(&format!(".function {name}{constructor}({params}) -> System.{name}\n{args}\ncall System.{name}::{constructor}({sig})\ncall instance System.Result<System.{name},System.Invalid{name}Error>::GetOkCase()\ncall instance System.Result.Ok<System.{name}>::get_Value()\nret\n.end\n"));
    }
    for (name, properties) in [
        (
            "Date",
            vec!["Year", "Month", "Day", "DayNumber", "DayOfYear"],
        ),
        (
            "Time",
            vec![
                "Hour",
                "Minute",
                "Second",
                "Millisecond",
                "FractionTicks",
                "Ticks",
            ],
        ),
    ] {
        for property in properties {
            let ty = if property == "Ticks" {
                "Int64"
            } else {
                "Int32"
            };
            source.push_str(&format!(".function {name}{property}(System.{name} value) -> {ty}\nldarga value\ncall instance System.{name}::get_{property}()\nret\n.end\n"));
        }
        for (method, ret) in [("CompareTo", "Int32"), ("Equals", "Boolean")] {
            source.push_str(&format!(".function {name}{method}(System.{name} left, System.{name} right) -> {ret}\nldarga left\nldarg right\ncall instance System.{name}::{method}(System.{name})\nret\n.end\n"));
        }
    }
    let module = neoclr::assemble(&source).unwrap();
    let p = LoadedProgram::new(&neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap())
        .unwrap();
    p.verify().unwrap();
    p
}
fn call(p: &LoadedProgram, sig: &str, args: Vec<Value>) -> Value {
    p.resolve_function(&parse_function_ref(sig).unwrap())
        .unwrap()
        .invoke(args, Limits::default())
        .unwrap()
        .value
}
fn ints(values: &[i32]) -> Vec<Value> {
    values.iter().copied().map(Value::Int32).collect()
}

#[test]
fn gregorian_components_match_pinned_dotnet_cases() {
    let p = program();
    let cases: Vec<[i32; 5]> =
        serde_json::from_str(include_str!("fixtures/date-cases.json")).unwrap();
    for [year, month, day, number, ordinal] in cases {
        let date = call(
            &p,
            "DateCreate(Int32,Int32,Int32)",
            ints(&[year, month, day]),
        );
        assert_eq!(date, call(&p, "DateFromDayNumber(Int32)", ints(&[number])));
        for (property, expected) in [
            ("Year", year),
            ("Month", month),
            ("Day", day),
            ("DayNumber", number),
            ("DayOfYear", ordinal),
        ] {
            assert_eq!(
                call(
                    &p,
                    &format!("Date{property}(System.Date)"),
                    vec![date.clone()]
                ),
                Value::Int32(expected),
                "{year}-{month}-{day} {property}"
            );
        }
    }
}
#[test]
fn time_ticks_preserve_fraction_and_day_boundaries() {
    let p = program();
    for (ticks, hour, minute, second, fraction) in [
        (0, 0, 0, 0, 0),
        (1, 0, 0, 0, 1),
        (10_000, 0, 0, 0, 10_000),
        (10_000_000, 0, 0, 1, 0),
        (452_961_234_567, 12, 34, 56, 1_234_567),
        (863_999_999_999, 23, 59, 59, 9_999_999),
    ] {
        let time = call(&p, "TimeFromTicks(Int64)", vec![Value::Int64(ticks)]);
        assert_eq!(
            time,
            call(
                &p,
                "TimeCreate(Int32,Int32,Int32,Int32)",
                ints(&[hour, minute, second, fraction])
            )
        );
        assert_eq!(
            call(&p, "TimeTicks(System.Time)", vec![time.clone()]),
            Value::Int64(ticks)
        );
        for (property, expected) in [
            ("Hour", hour),
            ("Minute", minute),
            ("Second", second),
            ("FractionTicks", fraction),
            ("Millisecond", fraction / 10_000),
        ] {
            assert_eq!(
                call(
                    &p,
                    &format!("Time{property}(System.Time)"),
                    vec![time.clone()]
                ),
                Value::Int32(expected)
            );
        }
    }
}
#[test]
fn invalid_components_are_results_and_comparison_is_value_based() {
    let p = program();
    for (sig, cases) in [
        (
            "System.Date::Create(Int32,Int32,Int32)",
            vec![
                vec![0, 1, 1],
                vec![10000, 1, 1],
                vec![2024, 0, 1],
                vec![2024, 13, 1],
                vec![2024, 1, 0],
                vec![1900, 2, 29],
                vec![2023, 2, 29],
                vec![2024, 4, 31],
                vec![i32::MAX, i32::MIN, i32::MAX],
            ],
        ),
        (
            "System.Time::Create(Int32,Int32,Int32,Int32)",
            vec![
                vec![24, 0, 0, 0],
                vec![-1, 0, 0, 0],
                vec![0, 60, 0, 0],
                vec![0, 0, 60, 0],
                vec![0, 0, 0, -1],
                vec![0, 0, 0, 10000000],
            ],
        ),
        (
            "System.Date::FromDayNumber(Int32)",
            vec![vec![-1], vec![3652059]],
        ),
    ] {
        for values in cases {
            let result = call(&p, sig, ints(&values));
            assert!(format!("{result:?}").contains("System.Result.Error"));
        }
    }
    for ticks in [-1, 864_000_000_000, i64::MAX] {
        assert!(
            format!(
                "{:?}",
                call(
                    &p,
                    "System.Time::FromTicks(Int64)",
                    vec![Value::Int64(ticks)]
                )
            )
            .contains("System.Result.Error")
        );
    }
    for (name, sig, left, right) in [
        (
            "Date",
            "DateFromDayNumber(Int32)",
            Value::Int32(0),
            Value::Int32(3652058),
        ),
        (
            "Time",
            "TimeFromTicks(Int64)",
            Value::Int64(0),
            Value::Int64(863999999999),
        ),
    ] {
        let a = call(&p, sig, vec![left]);
        let b = call(&p, sig, vec![right]);
        assert_eq!(
            call(
                &p,
                &format!("{name}CompareTo(System.{name},System.{name})"),
                vec![a.clone(), b.clone()]
            ),
            Value::Int32(-1)
        );
        assert_eq!(
            call(
                &p,
                &format!("{name}CompareTo(System.{name},System.{name})"),
                vec![b, a.clone()]
            ),
            Value::Int32(1)
        );
        assert_eq!(
            call(
                &p,
                &format!("{name}Equals(System.{name},System.{name})"),
                vec![a.clone(), a]
            ),
            Value::Boolean(true)
        );
    }
}
#[test]
fn neo_sample_defaults_and_results_roundtrip() {
    let module =
        neoclr::frontend::compile(include_str!("../examples/source/date-time.neo")).unwrap();
    let p = LoadedProgram::new(&neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap())
        .unwrap();
    p.verify().unwrap();
    let execution = p.run(Limits::default()).unwrap();
    assert_eq!(execution.value, Value::Int32(42));
    assert_eq!(execution.output, ["2024", "23", "InvalidDate"]);
}
#[test]
fn guest_cannot_forge_private_storage_even_without_verification() {
    for (name, scalar, constant) in [
        ("Date", "Int32", "ldc.i4 -1"),
        ("Time", "Int64", "ldc.i8 -1"),
    ] {
        assert!(neoclr::assemble(&format!(".module App\n.function Bad() -> System.{name}\n{constant}\nnewobj System.{name}\nret\n.end")).is_err());
        for (ret, body) in [
            (scalar.to_owned(), "ldfld 0\nret".to_owned()),
            (
                format!("System.{name}"),
                format!("{constant}\nstfld 0\nret"),
            ),
            (format!("{scalar}&"), "ldflda 0\nret".to_owned()),
        ] {
            let source = format!(
                ".module App\n.entry Main\n.function Main() -> {ret}\n.local System.{name} value\nldloca value\ninitobj System.{name}\nldloca value\n{body}\n.end"
            );
            let module = neoclr::assemble(&source).unwrap();
            let p = LoadedProgram::new(&module).unwrap();
            assert!(p.verify().is_err());
            let fault = p.run(Limits::default()).unwrap_err();
            assert!(fault.message.contains("field access denied"), "{fault}");
        }
    }
}
