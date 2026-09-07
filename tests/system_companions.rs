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
