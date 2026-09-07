use neoclr::{Limits, LoadedProgram, Value, assembler::parse_type};

#[test]
fn system_companions_are_ordinary_nested_types() {
    let source = ".module App\n.entry Main\n.function Main() -> Int32\nldc.i4 42\nnewobj instance System.Result.Ok<Int32>::.ctor(Int32)\ncall instance System.Result.Ok<Int32>::get_Value()\nret\n.end";
    let module = neoclr::assembler::assemble_modules(&[source]).unwrap();
    let program =
        LoadedProgram::with_library(&module[0], neoclr::library::system().unwrap()).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
    let identity = program
        .resolve_type_identity(&parse_type("System.Result.Ok<Int32>").unwrap())
        .unwrap();
    assert!(matches!(identity, neoclr::TypeIdentity::Definition { .. }));
}
