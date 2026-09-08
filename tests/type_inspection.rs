use neoclr::{
    Limits, LoadedProgram, RuntimeService, Value, assemble,
    assembler::{parse_function_ref, parse_type},
    load,
};

fn execute(body: &str, returns: &str) -> Result<Value, neoclr::Fault> {
    let module = assemble(&format!(
        ".module App\n.type Box<T>\n.field Value T\n.end\n.entry Main\n.function Main() -> {returns}\n{body}\nret\n.end"
    ))?;
    let module = load(&serde_json::to_string(&module).unwrap())?;
    let program = LoadedProgram::new(&module)?;
    program.verify()?;
    Ok(program.run(Limits::default())?.value)
}

#[test]
fn sample_and_reachability_use_metadata_without_erased_storage() {
    let module = assemble(include_str!("../examples/type_inspection.neoil")).unwrap();
    let program =
        LoadedProgram::new(&load(&serde_json::to_string(&module).unwrap()).unwrap()).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().output,
        ["System.Int32", "Box", "1", "System.Int32", "Same type"]
    );
    let graph = program
        .analyze_reachability(&[parse_function_ref("Main()").unwrap()], 30)
        .unwrap();
    assert!(
        graph
            .required_services()
            .contains(&RuntimeService::TypeInspection)
    );
    assert!(
        !graph
            .required_services()
            .contains(&RuntimeService::ValueStorage)
    );
}

#[test]
fn descriptors_use_closed_identity_and_preserve_declared_small_value_types() {
    for (left, right, same) in [
        ("int", "[System]System.Int32", true),
        ("Box<Byte>", "Box<Int32>", false),
        ("Void*", "Int32*", false),
    ] {
        let body = format!(
            ".local System.Type left\nldtoken {left}\ncall System.Type::GetTypeFromHandle(System.RuntimeTypeHandle)\nstloc left\nldloca left\nldtoken {right}\ncall System.Type::GetTypeFromHandle(System.RuntimeTypeHandle)\ncall instance System.Type::Equals(System.Type)"
        );
        assert_eq!(execute(&body, "Boolean").unwrap(), Value::Boolean(same));
    }
    for (ty, value, name) in [
        ("Byte", "ldc.i4 257", "System.Byte"),
        ("Void", "ldvoid", "System.Void"),
        ("Int32*", "ptr.null Int32", "System.Int32*"),
    ] {
        let body = format!(
            "{value}\ncall System.TypeOf<{ty}>::Of({ty})\ncall instance System.Type::get_Name()"
        );
        assert_eq!(
            execute(&body, "String").unwrap(),
            Value::String(name.into())
        );
    }
}

#[test]
fn generic_argument_queries_work_for_nested_and_pointer_arguments() {
    let prefix = "ldtoken Box<Box<Int32*>>\ncall System.Type::GetTypeFromHandle(System.RuntimeTypeHandle)\nldc.i4 0\ncall instance System.Type::GetGenericArgument(Int32)\nldc.i4 0\ncall instance System.Type::GetGenericArgument(Int32)";
    assert_eq!(
        execute(
            &format!("{prefix}\ncall instance System.Type::get_Name()"),
            "String"
        )
        .unwrap(),
        Value::String("System.Int32*".into())
    );
    for index in [-1, 1] {
        let fault = execute(&format!("ldtoken Box<Int32>\ncall System.Type::GetTypeFromHandle(System.RuntimeTypeHandle)\nldc.i4 {index}\ncall instance System.Type::GetGenericArgument(Int32)"), "System.Type").unwrap_err();
        assert!(
            fault
                .message
                .contains("generic argument index out of range")
        );
        assert!(fault.stack_trace.is_some());
    }
}

#[test]
fn handles_are_owned_after_execution_and_do_not_define_identity_by_name() {
    fn descriptor(module_name: &str, revision: &str) -> Box<neoclr::TypeDescriptor> {
        let source = format!(
            ".module {module_name}\n.revision {revision}\n.type Item\n.end\n.entry Main\n.function Main() -> System.RuntimeTypeHandle\nldtoken Item\nret\n.end"
        );
        let program = LoadedProgram::new(&assemble(&source).unwrap()).unwrap();
        let expected = program.describe_type(&parse_type("Item").unwrap()).unwrap();
        let value = program.run(Limits::default()).unwrap().value;
        drop(program);
        let Value::RuntimeTypeHandle(handle) = value else {
            panic!("expected handle")
        };
        assert_eq!(*handle, expected);
        handle
    }
    let first = descriptor("First", "one");
    let second = descriptor("Second", "one");
    let revised = descriptor("First", "two");
    assert_eq!(first.name, second.name);
    assert_ne!(first.identity, second.identity);
    assert_ne!(first.identity, revised.identity);
}

#[test]
fn invalid_tokens_layouts_and_host_handle_imports_are_rejected() {
    for body in [
        "ldtoken Missing",
        "ldtoken !0",
        "ldtoken Box",
        "ldtoken Box<Int32,Byte>",
        "sizeof System.RuntimeTypeHandle",
        "newobj System.RuntimeTypeHandle",
    ] {
        assert!(execute(body, "System.RuntimeTypeHandle").is_err(), "{body}");
    }
    let source = ".module App\n.function Echo(System.Type type) -> System.Type\nldarg type\nret\n.end\n.function Erased(System.Value value) -> Void\nldvoid\nret\n.end";
    let program = LoadedProgram::new(&assemble(source).unwrap()).unwrap();
    assert!(
        program
            .resolve_function(&parse_function_ref("Echo(System.Type)").unwrap())
            .unwrap_err()
            .message
            .contains("cannot be imported")
    );
    let handle = execute("ldtoken Int32", "System.RuntimeTypeHandle").unwrap();
    let echo = program
        .resolve_function(&parse_function_ref("Erased(System.Value)").unwrap())
        .unwrap();
    let fault = echo
        .invoke(vec![Value::Erased(Box::new(handle))], Limits::default())
        .unwrap_err();
    assert!(fault.message.contains("cannot be imported"));
    assert!(fault.stack_trace.is_none());
}

#[test]
fn token_metadata_does_not_bypass_visibility_or_module_references() {
    let library = ".module Models\n.type internal Hidden\n.end\n.type Public\n.method private static Secret() -> Void\nldvoid\nret\n.end\n.end";
    assemble(library).unwrap();
    neoclr::assembler::assemble_modules(&[".module App\n.references (Models)\n.function Main() -> System.RuntimeTypeHandle\nldtoken [Models]Public\nret\n.end", library]).unwrap();
    for body in [
        "ldtoken [Models]Hidden\npop\nldvoid",
        "ldtoken [Models]Public\npop\ncall Public::Secret()",
    ] {
        let app = format!(
            ".module App\n.references (Models)\n.function Main() -> Void\n{body}\nret\n.end"
        );
        assert!(neoclr::assembler::assemble_modules(&[&app, library]).is_err());
    }
    let app = ".module App\n.references ()\n.function Main() -> System.RuntimeTypeHandle\nldtoken [Models]Public\nret\n.end";
    assert!(neoclr::assembler::assemble_modules(&[app, library]).is_err());
}
