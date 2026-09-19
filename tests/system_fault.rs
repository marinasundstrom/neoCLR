use neoclr::{Limits, LoadedProgram, Value, assemble};

#[test]
fn dynamic_fault_stops_guest_but_does_not_abort_host() {
    let library = assemble(concat!(
        ".module System\n",
        include_str!("../runtime/raven/generated/Fault.methods.neoil"),
        include_str!("../runtime/raven/generated/Fault.helpers.neoil"),
        include_str!("../runtime/neoCLR/Runtime/Fault.neoil")
    ))
    .unwrap();
    for message in [
        "",
        "Query iterator has no current element",
        "failure: åäö ☃",
    ] {
        let app = assemble(&format!(
            r#".module Test
.entry Main
.function Main() -> Int32
ldstr "{message}"
call System.Fault(String)
ldc.i4 42
ret
.end
"#
        ))
        .unwrap();
        let program = LoadedProgram::with_library(&app, &library).unwrap();
        program.verify().unwrap();
        let fault = program.run(Limits::default()).unwrap_err();
        assert_eq!(fault.message, message);
    }
    let app =
        assemble(".module Healthy\n.entry Main\n.function Main() -> Int32\nldc.i4 42\nret\n.end")
            .unwrap();
    assert_eq!(
        LoadedProgram::with_library(&app, &library)
            .unwrap()
            .run(Limits::default())
            .unwrap()
            .value,
        Value::Int32(42)
    );
}

#[test]
fn fault_binding_requires_void_return_and_string_argument() {
    for (signature, expected) in [
        ("(Int32 message) -> Void", "no runtime binding"),
        ("(String message) -> Int32", "return type mismatch"),
        ("(String message) -> void", "no-result methods"),
    ] {
        let source = format!(
            ".module System\n.function neoCLR.Runtime.Fault{signature}\n.methodimpl InternalCall\n.end"
        );
        let fault = assemble(&source).unwrap_err();
        assert!(fault.message.contains(expected), "{}", fault.message);
    }
}
