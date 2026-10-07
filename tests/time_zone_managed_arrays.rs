use neoclr::{ExecutionOptions, LoadedProgram, Value, assembler::parse_function_ref};

#[test]
fn local_mapping_returns_managed_arrays_with_limits_and_exact_element_type() {
    let source = r#".module System
.function neoCLR.Runtime.TimeZoneMapLocal(String, Int64) -> arrayref<Int64>
.methodimpl InternalCall
.end
.function Read() -> Int32
ldstr "UTC"
ldc.i8 0
call neoCLR.Runtime.TimeZoneMapLocal(String, Int64)
ldlen
conv.i4
ret
.end
"#;
    let module = neoclr::assemble(source).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    let read = program
        .resolve_function(&parse_function_ref("Read()").unwrap())
        .unwrap();
    assert_eq!(
        read.invoke(vec![], ExecutionOptions::default())
            .unwrap()
            .value,
        Value::Int32(2)
    );
    let mut options = ExecutionOptions::default();
    options.limits.array_elements = 1;
    assert!(
        read.invoke(vec![], options)
            .unwrap_err()
            .message
            .contains("array")
    );
    let mut options = ExecutionOptions::default();
    options.limits.heap_objects = 0;
    assert!(
        read.invoke(vec![], options)
            .unwrap_err()
            .message
            .contains("heap")
    );
    for wrong in ["arrayref<Int32>", "arrayref<String>", "noresult"] {
        assert!(
            neoclr::assemble(&source.replace("-> arrayref<Int64>", &format!("-> {wrong}")))
                .is_err()
        );
    }
}
