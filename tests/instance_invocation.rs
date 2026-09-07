use neoclr::{
    Limits, LoadedProgram, Value, assemble, assembler::parse_function_ref, metadata::Type,
};

fn boxed(ty: Type, value: Value) -> Value {
    Value::Object {
        ty: Type::Constructed {
            definition: "Box".into(),
            arguments: vec![ty],
        },
        fields: vec![value],
    }
}

#[test]
fn generic_receivers_are_copied_and_returned_records_can_be_reused() {
    let module = assemble(include_str!("../examples/instance_invocation.neoil")).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    let update = program
        .resolve_function(&parse_function_ref("instance Box<Byte>::WithValue(Byte)").unwrap())
        .unwrap();
    let get = program
        .resolve_function(&parse_function_ref("instance Box<Byte>::Get()").unwrap())
        .unwrap();
    let original = boxed(Type::Byte, Value::Byte(1));
    assert_eq!(get.receiver_type(), Some(&original.ty()));
    assert!(get.parameters().is_empty());
    assert_eq!(update.parameters(), &[Type::Byte]);
    let result = update
        .invoke_instance(original.clone(), vec![Value::Byte(42)], Limits::default())
        .unwrap()
        .value;
    assert_eq!(original, boxed(Type::Byte, Value::Byte(1)));
    assert_eq!(
        get.invoke_instance(
            result,
            vec![],
            Limits {
                frames: 1,
                instructions: 3,
                ..Limits::default()
            }
        )
        .unwrap()
        .value,
        Value::Byte(42)
    );
    assert_eq!(
        get.invoke_instance(original, vec![], Limits::default())
            .unwrap()
            .value,
        Value::Byte(1)
    );
}

#[test]
fn receiver_and_argument_faults_are_separate_and_precede_execution() {
    let module = assemble(include_str!("../examples/instance_invocation.neoil")).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let update = program
        .resolve_function(&parse_function_ref("instance Box<Byte>::WithValue(Byte)").unwrap())
        .unwrap();
    let valid = boxed(Type::Byte, Value::Byte(1));
    for receiver in [
        Value::Byte(1),
        boxed(Type::Int32, Value::Int32(1)),
        boxed(Type::Byte, Value::Int32(1)),
        Value::Object {
            ty: valid.ty(),
            fields: vec![],
        },
    ] {
        let fault = update
            .invoke_instance(
                receiver,
                vec![Value::Byte(2)],
                Limits {
                    instructions: 0,
                    ..Limits::default()
                },
            )
            .unwrap_err();
        assert!(fault.message.starts_with("invocation receiver:"), "{fault}");
        assert!(fault.function.is_some());
        assert_eq!(fault.instruction, None);
    }
    let fault = update
        .invoke_instance(valid.clone(), vec![Value::Int32(2)], Limits::default())
        .unwrap_err();
    assert!(fault.message.starts_with("invocation argument 0:"));
    let fault = update
        .invoke_instance(valid, vec![], Limits::default())
        .unwrap_err();
    assert!(fault.message.contains("expected 1 arguments, got 0"));
}

#[test]
fn call_kind_is_explicit_in_safe_and_native_entry_points() {
    let module = assemble(".module App\n.type Point\n.method instance Echo() -> Point\nldarg this\nret\n.end\n.end\n.function Echo(Int32 x) -> Int32\nldarg x\nret\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let instance = program
        .resolve_function(&parse_function_ref("instance Point::Echo()").unwrap())
        .unwrap();
    let static_fn = program
        .resolve_function(&parse_function_ref("Echo(Int32)").unwrap())
        .unwrap();
    assert_eq!(static_fn.receiver_type(), None);
    assert!(
        instance
            .invoke(vec![], Limits::default())
            .unwrap_err()
            .message
            .contains("explicit receiver")
    );
    assert!(
        static_fn
            .invoke_instance(Value::Int32(1), vec![Value::Int32(2)], Limits::default())
            .unwrap_err()
            .message
            .contains("does not accept")
    );
    // No native declarations are present; the same call-kind checks apply.
    unsafe {
        assert!(
            instance
                .invoke_with_native(vec![], Limits::default())
                .is_err()
        );
        assert!(
            static_fn
                .invoke_instance_with_native(
                    Value::Int32(1),
                    vec![Value::Int32(2)],
                    Limits::default()
                )
                .is_err()
        );
        let receiver = Value::Object {
            ty: Type::Named("Point".into()),
            fields: vec![],
        };
        assert_eq!(
            instance
                .invoke_instance_with_native(receiver.clone(), vec![], Limits::default())
                .unwrap()
                .value,
            receiver
        );
    }
}

#[test]
fn primitive_receivers_use_existing_methods_and_guest_budgets() {
    let module = assemble(".module App").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let tostring = program
        .resolve_function(&parse_function_ref("instance System.Int32::ToString()").unwrap())
        .unwrap();
    assert_eq!(tostring.receiver_type(), Some(&Type::Int32));
    assert_eq!(
        tostring
            .invoke_instance(Value::Int32(42), vec![], Limits::default())
            .unwrap()
            .value,
        Value::String("42".into())
    );
    assert!(
        tostring
            .invoke_instance(
                Value::Int32(42),
                vec![],
                Limits {
                    instructions: 0,
                    ..Limits::default()
                }
            )
            .is_err()
    );
    assert!(
        tostring
            .invoke_instance(Value::Int32(1), vec![], Limits::default())
            .is_ok()
    );
}

#[test]
fn scoped_receivers_normalize_and_unsupported_stored_fields_fail_resolution() {
    let module = assemble(include_str!("../examples/instance_invocation.neoil")).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let get = program
        .resolve_function(
            &parse_function_ref("instance [InstanceInvocation]Box<Void>::Get()").unwrap(),
        )
        .unwrap();
    let scoped = |module: &str| Value::Object {
        ty: Type::Scoped {
            module: module.into(),
            name: "Box".into(),
            arguments: vec![Type::Void],
        },
        fields: vec![Value::Void],
    };
    assert_eq!(
        get.invoke_instance(scoped("InstanceInvocation"), vec![], Limits::default())
            .unwrap()
            .value,
        Value::Void
    );
    assert!(
        get.invoke_instance(scoped("Wrong"), vec![], Limits::default())
            .is_err()
    );
    for ty in ["Int32*", "Int32&"] {
        assert!(
            program
                .resolve_function(
                    &parse_function_ref(&format!("instance Box<{ty}>::Get()")).unwrap()
                )
                .is_err()
        );
    }
}
