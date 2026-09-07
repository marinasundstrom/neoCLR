use neoclr::{
    Limits, LoadedProgram, Value, assemble, assembler::parse_function_ref, metadata::Type,
};

const SOURCE: &str = ".module App\n.references ()\n.type Point\n.field X Int32\n.field Label String\n.end\n.function Update(Point point, Int32 x) -> Point\nldarg point\nldarg x\nstfld Point::X\nret\n.end\n.function Read(Point point) -> Int32\nldarg point\nldfld Point::X\nret\n.end";
fn point(x: i32) -> Value {
    Value::Object {
        ty: Type::Named("Point".into()),
        fields: vec![Value::Int32(x), Value::String("point".into())],
    }
}

#[test]
fn records_are_imported_by_value_and_results_can_be_imported_again() {
    let module = assemble(SOURCE).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let update = program
        .resolve_function(&parse_function_ref("Update(Point,Int32)").unwrap())
        .unwrap();
    let read = program
        .resolve_function(&parse_function_ref("Read(Point)").unwrap())
        .unwrap();
    let original = point(1);
    let result = update
        .invoke(vec![original.clone(), Value::Int32(42)], Limits::default())
        .unwrap()
        .value;
    assert_eq!(original, point(1));
    assert_eq!(result, point(42));
    assert_eq!(
        read.invoke(vec![result], Limits::default()).unwrap().value,
        Value::Int32(42)
    );
    assert_eq!(
        read.invoke(vec![original], Limits::default())
            .unwrap()
            .value,
        Value::Int32(1)
    );
}

#[test]
fn field_count_tags_and_concrete_field_shapes_are_checked_before_execution() {
    let module = assemble(SOURCE).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let read = program
        .resolve_function(&parse_function_ref("Read(Point)").unwrap())
        .unwrap();
    for value in [
        Value::Object {
            ty: Type::Named("Point".into()),
            fields: vec![],
        },
        Value::Object {
            ty: Type::Named("Point".into()),
            fields: vec![Value::Int32(1), Value::String("p".into()), Value::Void],
        },
        Value::Object {
            ty: Type::Named("Other".into()),
            fields: vec![Value::Int32(1), Value::String("p".into())],
        },
        Value::Object {
            ty: Type::Named("Point".into()),
            fields: vec![
                Value::Object {
                    ty: Type::Int32,
                    fields: vec![],
                },
                Value::String("p".into()),
            ],
        },
        Value::Int32(1),
    ] {
        let fault = read.invoke(vec![value], Limits::default()).unwrap_err();
        assert_eq!(fault.function.as_deref(), Some("Read"));
        assert_eq!(fault.instruction, None);
    }
    assert!(read.invoke(vec![point(42)], Limits::default()).is_ok());
}

#[test]
fn scoped_record_tags_are_checked_and_normalized() {
    let module = assemble(SOURCE).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let update = program
        .resolve_function(&parse_function_ref("Update(Point,Int32)").unwrap())
        .unwrap();
    let scoped = |module: &str| Value::Object {
        ty: Type::Scoped {
            module: module.into(),
            name: "Point".into(),
            arguments: vec![],
        },
        fields: vec![Value::Int32(1), Value::String("point".into())],
    };
    assert_eq!(
        update
            .invoke(vec![scoped("App"), Value::Int32(42)], Limits::default())
            .unwrap()
            .value,
        point(42)
    );
    assert!(
        update
            .invoke(vec![scoped("Wrong"), Value::Int32(42)], Limits::default())
            .is_err()
    );
}

#[test]
fn nested_closed_generics_retain_exact_primitive_fields_and_void() {
    let module = assemble(".module App\n.type Box<T>\n.field Value T\n.end\n.function Echo(Box<Box<Byte>> value) -> Box<Box<Byte>>\nldarg value\nret\n.end\n.function Unit(Box<Void> value) -> Box<Void>\nldarg value\nret\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let wrap = |ty, value| Value::Object {
        ty: Type::Constructed {
            definition: "Box".into(),
            arguments: vec![ty],
        },
        fields: vec![value],
    };
    let inner = wrap(Type::Byte, Value::Byte(7));
    let value = wrap(inner.ty(), inner);
    let echo = program
        .resolve_function(&parse_function_ref("Echo(Box<Box<Byte>>)").unwrap())
        .unwrap();
    assert_eq!(
        echo.invoke(vec![value.clone()], Limits::default())
            .unwrap()
            .value,
        value
    );
    let bad_inner = wrap(Type::Byte, Value::Int32(7));
    let fault = echo
        .invoke(vec![wrap(bad_inner.ty(), bad_inner)], Limits::default())
        .unwrap_err();
    assert!(fault.message.contains("field 0: field 0:"));
    let unit = program
        .resolve_function(&parse_function_ref("Unit(Box<Void>)").unwrap())
        .unwrap();
    let value = wrap(Type::Void, Value::Void);
    assert_eq!(
        unit.invoke(vec![value.clone()], Limits::default())
            .unwrap()
            .value,
        value
    );
}

#[test]
fn stored_pointers_and_refs_are_rejected_even_in_nested_records() {
    for field in ["Int32*", "Int32&"] {
        let source = format!(
            ".module App\n.type Inner\n.field Value {field}\n.end\n.type Outer\n.field Value Inner\n.end\n.function Ignore(Outer value) -> Void\nldvoid\nret\n.end"
        );
        let module = assemble(&source).unwrap();
        let program = LoadedProgram::new(&module).unwrap();
        assert!(
            program
                .resolve_function(&parse_function_ref("Ignore(Outer)").unwrap())
                .is_err(),
            "{field}"
        );
    }
    // A phantom type argument is not a stored pointer.
    let module = assemble(".module App\n.type Empty<T>\n.end\n.function Ignore(Empty<Int32*> value) -> Void\nldvoid\nret\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    assert!(
        program
            .resolve_function(&parse_function_ref("Ignore(Empty<Int32*>)").unwrap())
            .is_ok()
    );
}

#[test]
fn by_value_recursion_and_deep_schemas_fail_during_resolution() {
    for source in [
        ".module App\n.type Loop\n.field Next Loop\n.end\n.function F(Loop value) -> Void\nldvoid\nret\n.end".to_string(),
        format!(".module App\n{}\n.function F(N0 value) -> Void\nldvoid\nret\n.end", (0..66).map(|i| format!(".type N{i}\n.field Value {}\n.end", if i == 65 { "Int32".into() } else { format!("N{}", i+1) })).collect::<Vec<_>>().join("\n")),
    ] {
        let module = assemble(&source).unwrap();
        let parameter = module.functions[0].parameters[0].clone();
        let program = LoadedProgram::new(&module).unwrap();
        let mut target = parse_function_ref("F(Int32)").unwrap(); target.parameters[0] = parameter;
        assert!(program.resolve_function(&target).is_err());
    }
}
