use neoclr::{Limits, LoadedProgram, Value, assembler::assemble_modules};

#[test]
fn union_extraction_uses_discriminator_then_typed_accessor() {
    let source = include_str!("../examples/union_extract.neoil");
    let modules = assemble_modules(&[source]).unwrap();
    let program =
        LoadedProgram::with_library(&modules[0], neoclr::library::system().unwrap()).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}
