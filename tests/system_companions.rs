use neoclr::{Limits, LoadedProgram, Value, assembler::assemble_modules};

#[test]
fn nested_system_case_is_constructed_and_read_like_an_ordinary_type() {
    let source = include_str!("../examples/system_companions.neoil");
    let modules = assemble_modules(&[source]).unwrap();
    let program =
        LoadedProgram::with_library(&modules[0], neoclr::library::system().unwrap()).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn type_descriptor_exposes_name_arguments_and_owner_read_only() {
    let source = ".module App\n.entry Main\n.function Main() -> Void\nldvoid\nret\n.end";
    let modules = assemble_modules(&[source]).unwrap();
    let program =
        LoadedProgram::with_library(&modules[0], neoclr::library::system().unwrap()).unwrap();
    let descriptor = program
        .describe_type(&neoclr::assembler::parse_type("System.Result.Ok<Int32>").unwrap())
        .unwrap();
    assert_eq!(descriptor.name, "System.Result.Ok");
    assert_eq!(descriptor.generic_arguments.len(), 1);
    assert_eq!(descriptor.generic_arguments[0].name, "System.Int32");
    assert!(descriptor.declaring_type.is_some());
}
