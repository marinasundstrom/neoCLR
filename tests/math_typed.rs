use neoclr::{Limits, LoadedProgram, Value, assembler::assemble_modules};

#[test]
fn typed_abs_uses_nested_result_cases() {
    let source = ".module App\n.entry Main\n.function Main() -> Int32\nldc.i4 -42\ncall System.Math.Abs(Int32)\ncall instance System.Result<Int32,Error>::GetOkCase()\ncall instance System.Result.Ok<Int32>::get_Value()\nret\n.end";
    let modules = assemble_modules(&[source]).unwrap();
    let program =
        LoadedProgram::with_library(&modules[0], neoclr::library::system().unwrap()).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}
