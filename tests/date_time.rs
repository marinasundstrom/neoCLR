include!("support/calendar_suite.rs");

fn load_calendar_program(source: &str) -> LoadedProgram {
    let module = neoclr::assemble(source).unwrap();
    LoadedProgram::new(&neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap()).unwrap()
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
