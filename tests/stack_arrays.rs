use neoclr::{Limits, LoadedProgram, assemble};

#[test]
fn localloc_can_back_an_array_descriptor_for_frame_lifetime() {
    let module = assemble(include_str!("../examples/arrays_stack.neoil")).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    let result = program.run(Limits::default()).unwrap();
    assert_eq!(result.output, ["42"]);
}
