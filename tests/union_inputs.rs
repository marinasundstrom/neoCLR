use neoclr::{Limits, LoadedProgram, Value, assemble, assembler::parse_function_ref};
#[test]
fn host_reimports_ordinary_result_values_and_rejects_wrong_record_shapes() {
    let module = assemble(include_str!("../examples/ordinary_union_inputs.neoil")).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let create = program
        .resolve_function(&parse_function_ref("Create(Int32)").unwrap())
        .unwrap();
    let read = program
        .resolve_function(&parse_function_ref("Read(System.Result<Int32,String>)").unwrap())
        .unwrap();
    let result = create
        .invoke(vec![Value::Int32(42)], Limits::default())
        .unwrap()
        .value;
    assert_eq!(
        read.invoke(vec![result.clone()], Limits::default())
            .unwrap()
            .value,
        Value::Int32(42)
    );
    let Value::Object { ty, .. } = result else {
        panic!("expected ordinary carrier")
    };
    assert!(
        read.invoke(
            vec![Value::Object { ty, fields: vec![] }],
            Limits::default()
        )
        .is_err()
    );
}
#[test]
fn format_four_rejects_old_union_encodings_and_instructions() {
    for op in [
        "some",
        "none Int32",
        "ok Error",
        "err Int32",
        "is.case Some",
        "ldcase Ok",
    ] {
        assert!(
            assemble(&format!(
                ".module App\n.function F() -> Void\n{op}\nret\n.end"
            ))
            .is_err()
        );
    }
    for format in [1, 2, 3] {
        assert!(
            neoclr::load(&format!(
                r#"{{"format":{format},"name":"Old","entry":"","types":[],"functions":[]}}"#
            ))
            .is_err()
        );
    }
    for ty in [r#"{"Option":"Int32"}"#, r#"{"Result":["Int32","Error"]}"#] {
        assert!(serde_json::from_str::<neoclr::metadata::Type>(ty).is_err());
    }
}

#[test]
fn erased_case_storage_checks_pointer_payloads_at_import() {
    let module = assemble(".module App\n.function None() -> System.Option<Int32*>\nnewobj instance System.Option.None::.ctor()\nnewobj instance System.Option<Int32*>::.ctor(System.Option.None)\nret\n.end\n.function Some() -> System.Option<Int32*>\nptr.null Int32\nnewobj instance System.Option.Some<Int32*>::.ctor(Int32*)\nnewobj instance System.Option<Int32*>::.ctor(System.Option.Some<Int32*>)\nret\n.end\n.function Ignore(System.Option<Int32*> value) -> Void\nldvoid\nret\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let ignore = program
        .resolve_function(&parse_function_ref("Ignore(System.Option<Int32*>)").unwrap())
        .unwrap();
    for (name, accepted) in [("None()", true), ("Some()", false)] {
        let value = program
            .resolve_function(&parse_function_ref(name).unwrap())
            .unwrap()
            .invoke(vec![], Limits::default())
            .unwrap()
            .value;
        let result = ignore.invoke(vec![value], Limits::default());
        assert_eq!(result.is_ok(), accepted);
        if let Err(fault) = result {
            assert!(fault.stack_trace.is_none());
            assert!(fault.message.contains("pointer and Ref"), "{fault}");
        }
    }
}
