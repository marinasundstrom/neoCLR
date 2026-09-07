use neoclr::{
    Limits, LoadedProgram, RuntimeService, Value, assemble, assembler::parse_function_ref,
    metadata::Type,
};

fn run_body(body: &str, returns: &str, types: &str) -> Result<neoclr::Execution, neoclr::Fault> {
    let module = assemble(&format!(
        ".module App\n.entry Main\n{types}\n.function Main() -> {returns}\n{body}\n.end"
    ))?;
    let program = LoadedProgram::new(&module)?;
    program.verify()?;
    program.run(Limits::default())
}

#[test]
fn array_program_shares_explicit_storage_and_frees_it_once() {
    let module = assemble(include_str!("../examples/arrays.neoil")).unwrap();
    let json = serde_json::to_string(&module).unwrap();
    let program = LoadedProgram::new(&neoclr::load(&json).unwrap()).unwrap();
    program.verify().unwrap();
    let result = program.run(Limits::default()).unwrap();
    assert_eq!(result.output, ["10", "42", "10"]);
    assert_eq!(result.memory.live_allocations(), 0);
}

#[test]
fn native_element_types_keep_exact_storage_and_explicit_initial_values() {
    for (ty, initial, expected) in [
        ("Byte", "ldc.i4 255", Value::Byte(255)),
        ("Double", "ldc.r8 1.5", Value::Double(1.5)),
        ("Void", "ldvoid", Value::Void),
        (
            "Point",
            "ldc.i4 7\nnewobj Point",
            Value::Object {
                ty: Type::Named("Point".into()),
                fields: vec![Value::Int32(7)],
            },
        ),
    ] {
        let body = format!(
            ".local System.Array<{ty}> array\n.local {ty} result\nldc.i4 2\n{initial}\ncall System.Array<{ty}>::Allocate(Int32,{ty})\nstloc array\nldloc array\nldc.i4 1\ncall instance System.Array<{ty}>::Get(Int32)\nstloc result\nldloc array\ncall instance System.Array<{ty}>::Free()\npop\nldloc result\nret"
        );
        let result = run_body(&body, ty, ".type Point\n.field X Int32\n.end").unwrap();
        assert_eq!(result.value, expected);
        assert_eq!(result.memory.live_allocations(), 0);
    }
}

#[test]
fn record_elements_are_copied_on_get_and_set_only_changes_one_slot() {
    let result = run_body(".local System.Array<Point> array\n.local Point copy\nldc.i4 2\nldc.i4 1\nnewobj Point\ncall System.Array<Point>::Allocate(Int32,Point)\nstloc array\nldloc array\nldc.i4 0\ncall instance System.Array<Point>::Get(Int32)\nldc.i4 9\nstfld Point::X\nstloc copy\nldloc array\nldc.i4 0\ncall instance System.Array<Point>::Get(Int32)\nldfld Point::X\ncall System.Console::WriteLine(Int32)\npop\nldloc array\nldc.i4 0\nldloc copy\ncall instance System.Array<Point>::Set(Int32,Point)\npop\nldloc array\nldc.i4 1\ncall instance System.Array<Point>::Get(Int32)\nldfld Point::X\ncall System.Console::WriteLine(Int32)\npop\nldloc array\nldc.i4 0\ncall instance System.Array<Point>::Get(Int32)\nldfld Point::X\ncall System.Console::WriteLine(Int32)\npop\nldloc array\ncall instance System.Array<Point>::Free()\nret", "Void", ".type Point\n.field X Int32\n.end").unwrap();
    assert_eq!(result.output, ["1", "1", "9"]);
    assert_eq!(result.memory.live_allocations(), 0);
}

#[test]
fn bounds_and_negative_length_faults_preserve_library_and_app_frames() {
    for (length, index) in [(0, 0), (2, -1), (2, 2)] {
        let fault = run_body(&format!("ldc.i4 {length}\nldc.i4 0\ncall System.Array<Int32>::Allocate(Int32,Int32)\nldc.i4 {index}\ncall instance System.Array<Int32>::Get(Int32)\nret"),"Int32", "").unwrap_err();
        assert_eq!(fault.message, "Array index out of range");
        let frames = fault.stack_trace.unwrap().frames;
        assert_eq!(
            frames
                .iter()
                .map(|f| f.function.name.as_str())
                .collect::<Vec<_>>(),
            ["System.Array.GetElementAddress", "System.Array.Get", "Main"]
        );
        assert_eq!(
            frames[0].function.owner,
            Some(neoclr::assembler::parse_type("System.Array<Int32>").unwrap())
        );
    }
    let fault = run_body(
        "ldc.i4 -1\nldc.i4 0\ncall System.Array<Int32>::Allocate(Int32,Int32)\npop\nldvoid\nret",
        "Void",
        "",
    )
    .unwrap_err();
    assert_eq!(fault.message, "Array length must be non-negative");
}

#[test]
fn empty_and_void_arrays_retain_length_without_element_bytes() {
    for length in [0, 3] {
        let result = run_body(&format!(".local System.Array<Void> array\n.local Int32 length\nldc.i4 {length}\nldvoid\ncall System.Array<Void>::Allocate(Int32,Void)\nstloc array\nldloc array\ncall instance System.Array<Void>::get_Length()\nstloc length\nldloc array\ncall instance System.Array<Void>::Free()\npop\nldloc length\nret"),"Int32", "").unwrap();
        assert_eq!(result.value, Value::Int32(length));
        assert_eq!(result.memory.live_bytes(), 0);
        assert_eq!(result.memory.live_allocations(), 0);
    }
}

#[test]
fn freed_buffers_are_invalid_through_every_descriptor_copy() {
    let prefix = ".local System.Array<Int32> array\n.local System.Array<Int32> alias\nldc.i4 1\nldc.i4 0\ncall System.Array<Int32>::Allocate(Int32,Int32)\nstloc array\nldloc array\nstloc alias\nldloc array\ncall instance System.Array<Int32>::Free()\npop\nldloc alias\n";
    for suffix in [
        "ldc.i4 0\ncall instance System.Array<Int32>::Get(Int32)\npop\nldvoid\nret",
        "call instance System.Array<Int32>::Free()\nret",
    ] {
        assert!(run_body(&format!("{prefix}{suffix}"), "Void", "").is_err());
    }
}

#[test]
fn layout_and_resource_limits_apply_and_planning_uses_existing_services() {
    let source = ".module App\n.entry Main\n.function Main() -> System.Array<Int32>\nldc.i4 2\nldc.i4 0\ncall System.Array<Int32>::Allocate(Int32,Int32)\nret\n.end";
    let module = assemble(source).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let fault = program
        .run(Limits {
            pointer_bytes: 4,
            ..Limits::default()
        })
        .unwrap_err();
    assert!(fault.message.contains("byte limit"));
    let graph = program
        .analyze_reachability(&[parse_function_ref("Main()").unwrap()], 8)
        .unwrap();
    assert_eq!(
        graph.required_services(),
        [
            RuntimeService::NativeAllocation,
            RuntimeService::PointerMemory,
            RuntimeService::SlotReferences
        ]
    );
    assert!(run_body("ldc.i4 0\nldstr \"\"\ncall System.Array<String>::Allocate(Int32,String)\npop\nldvoid\nret","Void", "").unwrap_err().message.contains("layout is not implemented"));
    // Native storage is owned by the Execution even when a descriptor is returned.
    let result = program.run(Limits::default()).unwrap();
    assert_eq!(result.memory.live_allocations(), 1);
    assert_eq!(result.memory.live_bytes(), 8);
}
