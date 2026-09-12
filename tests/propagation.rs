use neoclr::{Limits, LoadedProgram, Value, assemble, load};

fn execute(source: &str) -> neoclr::Execution {
    let module = assemble(source).unwrap();
    let module = load(&serde_json::to_string(&module).unwrap()).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    program.run(Limits::default()).unwrap()
}

#[test]
fn result_extracts_through_the_interface_without_overwriting_a_missed_output() {
    for (factory, expected) in [
        (
            "ldc.i4 42\ncall System.Result<Int32,String>::Ok(Int32)",
            vec!["true", "42", "false", "sentinel"],
        ),
        (
            "ldstr \"failure\"\ncall System.Result<Int32,String>::FromResidual(String)",
            vec!["false", "7", "true", "failure"],
        ),
    ] {
        let source = format!(
            r#"
.module Propagation
.entry Main
.function PrintBoolean(Boolean value) -> Void
    ldarg value
    brfalse False
    ldstr "true"
    call System.Console::WriteLine(String)
    ret
False:
    ldstr "false"
    call System.Console::WriteLine(String)
    ret
.end
.function Main() -> Void
    .local System.Result<Int32,String> carrier
    .local Int32 output
    .local String residual
    {factory}
    stloc carrier
    ldc.i4 7
    stloc output
    ldstr "sentinel"
    stloc residual
    ldloca carrier
    interface.borrow System.Propagatable<System.Result<Int32,String>,Int32,String>
    ldloca output
    callvirt instance System.Propagatable<System.Result<Int32,String>,Int32,String>::TryGetOutput(Int32&)
    call PrintBoolean(Boolean)
    pop
    ldloc output
    call System.Console::WriteLine(Int32)
    pop
    ldloca carrier
    interface.borrow System.Propagatable<System.Result<Int32,String>,Int32,String>
    ldloca residual
    callvirt instance System.Propagatable<System.Result<Int32,String>,Int32,String>::TryGetResidual(String&)
    call PrintBoolean(Boolean)
    pop
    ldloc residual
    call System.Console::WriteLine(String)
    ret
.end
"#
        );
        assert_eq!(execute(&source).output, expected);
    }
}

#[test]
fn option_void_is_success_or_absence_even_without_a_payload() {
    for (factory, success) in [
        (
            "ldvoid\nnewobj instance System.Option.Some<Void>::.ctor(Void)\nnewobj instance System.Option<Void>::.ctor(System.Option.Some<Void>)",
            true,
        ),
        (
            "ldvoid\ncall System.Option<Void>::FromResidual(Void)",
            false,
        ),
    ] {
        let source = format!(
            r#"
.module VoidPropagation
.entry Main
.function Main() -> Boolean
    .local System.Option<Void> carrier
    .local Void payload
    {factory}
    stloc carrier
    ldloca carrier
    interface.borrow System.Propagatable<System.Option<Void>,Void,Void>
    ldloca payload
    callvirt instance System.Propagatable<System.Option<Void>,Void,Void>::TryGetOutput(Void&)
    brfalse Absent
    ldloc payload
    pop
    ldc.bool true
    ret
Absent:
    ldloca carrier
    interface.borrow System.Propagatable<System.Option<Void>,Void,Void>
    ldloca payload
    callvirt instance System.Propagatable<System.Option<Void>,Void,Void>::TryGetResidual(Void&)
    brfalse Invalid
    ldloc payload
    pop
    ldc.bool false
    ret
Invalid:
    fault "carrier has neither output nor residual"
.end
"#
        );
        assert_eq!(execute(&source).value, Value::Boolean(success));
    }
}

#[test]
fn residual_factory_changes_output_type_and_preserves_the_error() {
    let execution = execute(
        r#"
.module Residual
.entry Main
.function Main() -> String
    .local System.Result<Int32,String> source
    .local String error
    ldstr "failure"
    call System.Result<Int32,String>::FromResidual(String)
    stloc source
    ldloca source
    ldloca error
    call instance System.Result<Int32,String>::TryGetResidual(String&)
    brfalse Invalid
    ldloc error
    call System.Result<Void,String>::FromResidual(String)
    call instance System.Result<Void,String>::GetErrorCase()
    call instance System.Result.Error<String>::get_Value()
    ret
Invalid:
    fault "expected error"
.end
"#,
    );
    assert_eq!(execution.value, Value::String("failure".into()));
}

#[test]
fn failed_extraction_does_not_prove_initialization() {
    let source = r#"
.module Uninitialized
.entry Main
.function Main() -> Int32
    .local System.Result<Int32,String> source
    .local Int32 output
    ldstr "failure"
    call System.Result<Int32,String>::FromResidual(String)
    stloc source
    ldloca source
    ldloca output
    call instance System.Result<Int32,String>::TryGetOutput(Int32&)
    pop
    ldloc output
    ret
.end
"#;
    let module = assemble(source).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    assert!(program.verify().is_err());
    assert!(program.run(Limits::default()).is_err());
}
