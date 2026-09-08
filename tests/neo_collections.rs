use neoclr::{Limits, LoadedProgram, Value, frontend};

#[test]
fn complete_collection_program_preserves_references_and_descriptor_copy_contract() {
    let module = frontend::compile(include_str!("../examples/source/collections.neo")).unwrap();
    let module = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    let result = LoadedProgram::new(&module)
        .unwrap()
        .run(Limits::default())
        .unwrap();
    assert_eq!(result.output, ["42", "1", "1", "2"]);
    assert_eq!(result.value, Value::Int32(42));
}

#[test]
fn generic_owner_parsing_preserves_comparisons_and_rejects_bad_conversions() {
    assert!(
        frontend::compile("func Main() -> bool { let a = 1\nlet b = 2\nreturn a < b }").is_ok()
    );
    for source in [
        "func Main() -> () { let a = System.Collections.ArrayList<int,string>.Allocate(0) }",
        "func Main() -> () { var a = System.Collections.ArrayList<int>.Allocate(0)\nlet b: System.Collections.List<string>& = &a }",
        "func Main() -> () { var a = System.Collections.ArrayList<int>.Allocate(0)\nlet b: System.Collections.List<int>& = a }",
    ] {
        assert!(frontend::compile(source).is_err(), "{source}");
    }
}
