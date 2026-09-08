use neoclr::{Limits, LoadedProgram, Value, assemble, assembler::parse_type, frontend, load};

fn execute(source: &str) -> Result<neoclr::Execution, neoclr::Fault> {
    let module = frontend::compile(source)?;
    let module = load(&serde_json::to_string(&module).unwrap())?;
    LoadedProgram::new(&module)?.run(Limits::default())
}

#[test]
fn neo_type_and_method_constraints_keep_value_and_reference_modes_separate() {
    let source = r#"
record ValueOnly<T>(Value: T) where T: notvoid, notreference
record ContainsReference(Value: int&)
func Identity<T>(value: T) -> T where T: notvoid, notreference { return value }
record Helpers() {
    static func Forward<T>(value: T) -> T where T: notvoid { return Identity<T>(value) }
}
func Main() -> int {
    let heap = new ValueOnly<int>(42)
    let reference = &heap.Value
    let composite = ValueOnly<ContainsReference>(ContainsReference(reference))
    let copied: int = composite.Value.Value
    return Helpers.Forward(copied)
}
"#;
    assert_eq!(execute(source).unwrap().value, Value::Int32(42));
}

#[test]
fn neo_rejects_invalid_concrete_arguments_and_symbolic_forwarding_cannot_bypass_checks() {
    let declarations = "record NonVoid<T>(Value: T) where T: notvoid\nrecord ValueOnly<T>(Value: T) where T: notreference\nfunc NonVoidCall<T>(value: T) -> T where T: notvoid { return value }\nfunc Forward<T>(value: T) -> T { return NonVoidCall<T>(value) }\nfunc Nothing() -> () {}\n";
    for body in [
        "let value = NonVoid<Void>(Nothing())",
        "var value = 42; let wrapped = ValueOnly<int&>(&value)",
        "var value = 42; let reference: readonly int& = &value; let wrapped = ValueOnly<readonly int&>(reference)",
        "Forward<Void>(Nothing())",
    ] {
        let error = execute(&format!("{declarations}func Main() -> () {{ {body} }}")).unwrap_err();
        assert!(error.message.contains("violates"), "{error:?}");
    }
}

#[test]
fn il_metadata_and_host_field_resolution_enforce_constraints_without_neo() {
    let module = assemble(".module Constraints\n.type Restricted<T>\n.constraint T notvoid notreference\n.field Value T\n.end\n.type Outer<T>\n.field Nested Restricted<T>\n.end").unwrap();
    let json = serde_json::to_string(&module).unwrap();
    assert!(json.contains("generic_constraints"));
    let module = load(&json).unwrap();
    assert!(
        module
            .instantiated_fields(&parse_type("Restricted<Int32>").unwrap())
            .is_ok()
    );
    for ty in [
        "Restricted<Void>",
        "Restricted<Int32&>",
        "Restricted<readonly Int32&>",
        "Outer<Void>",
    ] {
        assert!(
            module
                .instantiated_fields(&parse_type(ty).unwrap())
                .unwrap_err()
                .message
                .contains("violates"),
            "{ty}"
        );
    }
    // Outer form, not recursive graph restrictions: pointers and containers remain separate.
    assert!(
        module
            .instantiated_fields(&parse_type("Restricted<Void*>").unwrap())
            .is_ok()
    );
    let mut invalid: serde_json::Value = serde_json::from_str(&json).unwrap();
    invalid["types"][0]["generic_constraints"][0]["parameter"] = 4.into();
    assert!(
        load(&invalid.to_string())
            .unwrap_err()
            .message
            .contains("outside declaring context")
    );
}

#[test]
fn il_generic_calls_and_host_resolution_enforce_method_constraints() {
    let module = assemble(".module Constraints\n.function Id<T>(T value) -> T\n.constraint T notvoid notreference\nldarg value\nret\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let valid = neoclr::assembler::parse_function_ref("Id<Int32>(Int32)").unwrap();
    assert_eq!(
        program
            .resolve_function(&valid)
            .unwrap()
            .invoke(vec![Value::Int32(42)], Limits::default())
            .unwrap()
            .value,
        Value::Int32(42)
    );
    for signature in ["Id<Void>(Void)", "Id<Int32&>(Int32&)"] {
        let target = neoclr::assembler::parse_function_ref(signature).unwrap();
        assert!(
            program
                .resolve_function(&target)
                .err()
                .unwrap()
                .message
                .contains("violates")
        );
    }
}

#[test]
fn malformed_and_unimplemented_constraints_are_rejected() {
    for constraint in [
        "T notvoid notvoid",
        "Missing notvoid",
        "T notnull",
        "T",
        "T class",
    ] {
        assert!(
            assemble(&format!(
                ".module Bad\n.type Box<T>\n.constraint {constraint}\n.end"
            ))
            .is_err()
        );
    }
    for source in [
        "record Box<T>(Value: T) where T: notvoid, notvoid\nfunc Main() -> () {}",
        "record Box<T>(Value: T) where Missing: notvoid\nfunc Main() -> () {}",
        "func Id<T>(value: T) -> T where T: notnull { return value }\nfunc Main() -> () {}",
    ] {
        assert!(frontend::compile(source).is_err());
    }
}

#[test]
fn substituted_base_contracts_and_multiple_parameter_clauses_are_checked() {
    let declarations = ".module BaseConstraints\n.type Base<T>\n.constraint T notvoid\n.end\n.type Derived<T>\n.extends Base<T>\n.end\n";
    let module = assemble(declarations).unwrap();
    assert!(
        module
            .instantiated_fields(&parse_type("Derived<Void>").unwrap())
            .unwrap_err()
            .message
            .contains("violates")
    );
    assert!(
        module
            .instantiated_fields(&parse_type("Derived<Int32>").unwrap())
            .is_ok()
    );
    assert_eq!(
        execute(
            r#"
record Pair<T,U>(First: T, Second: U)
where T: notvoid
where U: notreference
func Main() -> int { return Pair<int,Void>(42, Nothing()).First }
func Nothing() -> () {}
"#
        )
        .unwrap()
        .value,
        Value::Int32(42)
    );
}

#[test]
fn constraint_example_runs() {
    let execution = execute(include_str!("../examples/source/generic-constraints.neo")).unwrap();
    assert_eq!(execution.value, Value::Int32(42));
    assert_eq!(execution.output, ["42"]);
}

#[test]
fn owner_and_method_parameter_constraints_have_independent_indices() {
    let module = assemble(".module Both\n.type Owner<T>\n.constraint 0 notreference\n.method static Id<U>(U value) -> U\n.constraint U notvoid\nldarg value\nret\n.end\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let valid = neoclr::assembler::parse_function_ref("Owner<Int32>::Id<Int32>(Int32)").unwrap();
    assert_eq!(
        program
            .resolve_function(&valid)
            .unwrap()
            .invoke(vec![Value::Int32(42)], Limits::default())
            .unwrap()
            .value,
        Value::Int32(42)
    );
    for (signature, restriction) in [
        ("Owner<Int32&>::Id<Int32>(Int32)", "NotReference"),
        ("Owner<Int32>::Id<Void>(Void)", "NotVoid"),
    ] {
        let target = neoclr::assembler::parse_function_ref(signature).unwrap();
        assert!(
            program
                .resolve_function(&target)
                .err()
                .unwrap()
                .message
                .contains(restriction)
        );
    }
}
