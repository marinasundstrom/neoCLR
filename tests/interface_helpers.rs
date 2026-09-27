use neoclr::{Limits, LoadedProgram, Value, assemble};

const SOURCE: &str = r#"
.module Helpers
.entry Main
.interface Report
.method static Offset() -> Int32
ldc.i4 2
ret
.end
.method private static Twice(Int32 value) -> Int32
ldarg value
ldc.i4 2
mul
ret
.end
.method instance Read() -> Int32
ldc.i4 20
call Report::Twice(Int32)
call Report::Offset()
add
ret
.end
.end
.type class Sample
.implements Report
.method instance .ctor() -> noresult
ret
.end
.end
.function Main() -> Int32
newobj instance Sample::.ctor()
callvirt instance Report::Read()
ret
.end
"#;

#[test]
fn nominal_default_calls_private_static_helper_without_implementation_obligation() {
    let module = assemble(SOURCE).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
    let graph = program
        .analyze_reachability(
            &[neoclr::assembler::parse_function_ref("Main()").unwrap()],
            100,
        )
        .unwrap();
    assert!(
        graph
            .functions
            .iter()
            .any(|f| f.target.name == "Report.Twice")
    );
    assert!(
        graph
            .functions
            .iter()
            .any(|f| f.target.name == "Report.Offset")
    );
}

#[test]
fn interface_helpers_preserve_access_and_direct_call_rules() {
    for changed in [
        SOURCE.replace(
            "newobj instance Sample::.ctor()\ncallvirt instance Report::Read()",
            "ldc.i4 20\ncall Report::Twice(Int32)",
        ),
        SOURCE.replace("call Report::Twice(Int32)", "callvirt Report::Twice(Int32)"),
        SOURCE.replace(
            "callvirt instance Report::Read()",
            "call instance Report::Read()",
        ),
        SOURCE.replace(
            ".method private static Twice",
            ".method private static virtual Twice",
        ),
        SOURCE.replace(
            ".method private static Twice",
            ".method private instance Twice",
        ),
    ] {
        let admitted = assemble(&changed).and_then(|module| {
            let program = LoadedProgram::new(&module)?;
            program.verify().map(|_| ())
        });
        assert!(
            admitted.is_err(),
            "invalid helper access or dispatch admitted"
        );
    }
}
