use neoclr::metadata::{Instruction as Op, StaticField, Type, Visibility};
use neoclr::{Limits, Value, assemble, run, verify};

fn owner(argument: Type) -> Type {
    Type::Constructed {
        definition: "Cache".into(),
        arguments: vec![argument],
    }
}
fn fixture(field_type: Type) -> neoclr::Module {
    let mut module = assemble(".module StaticStorage\n.entry Main\n.type class Cache<T>\n.end\n.type class Item\n.field Number Int32\n.method instance .ctor() -> void\nret\n.end\n.end\n.function Main() -> Int32\nldc.i4 0\nret\n.end").unwrap();
    module.types[0].static_fields.push(StaticField {
        name: "Value".into(),
        ty: field_type,
        visibility: Visibility::Public,
        readonly: false,
    });
    module
}
fn load(owner: Type) -> Op {
    Op::StaticField { owner, index: 0 }
}
fn store(owner: Type) -> Op {
    Op::SetStaticField { owner, index: 0 }
}
fn body(module: &mut neoclr::Module, ops: Vec<Op>) {
    module
        .functions
        .iter_mut()
        .find(|f| f.name == "Main")
        .unwrap()
        .body = ops;
}

#[test]
fn closed_generic_types_have_distinct_storage_and_executions_reset_it() {
    let mut module = fixture(Type::Int32);
    let a = owner(Type::Int32);
    let b = owner(Type::String);
    body(
        &mut module,
        vec![
            load(a.clone()),
            Op::Int(41),
            Op::Add,
            store(a.clone()),
            load(b.clone()),
            Op::Int(1),
            Op::Add,
            store(b.clone()),
            load(a),
            load(b),
            Op::Add,
            Op::Return,
        ],
    );
    verify(&module).unwrap();
    for _ in 0..2 {
        assert_eq!(
            run(&module, Limits::default()).unwrap().value,
            Value::Int32(42)
        );
    }
    let image = serde_json::to_vec(&module).unwrap();
    let reread: neoclr::Module = serde_json::from_slice(&image).unwrap();
    assert_eq!(
        run(&reread, Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn type_parameter_storage_is_substituted_per_closed_owner() {
    let mut module = fixture(Type::TypeParameter(0));
    body(
        &mut module,
        vec![
            Op::Int(42),
            store(owner(Type::Int32)),
            Op::String("other".into()),
            store(owner(Type::String)),
            load(owner(Type::Int32)),
            Op::Return,
        ],
    );
    verify(&module).unwrap();
    assert_eq!(
        run(&module, Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn static_references_survive_pressure_and_are_reclaimed_at_teardown() {
    let mut module = fixture(Type::Named("Item".into()));
    let constructor = neoclr::assembler::parse_function_ref("instance Item::.ctor()").unwrap();
    let mut ops = vec![
        Op::Construct(constructor.clone()),
        Op::Dup,
        Op::Int(42),
        Op::SetField(0),
        store(owner(Type::Int32)),
    ];
    for _ in 0..20 {
        ops.extend([Op::Construct(constructor.clone()), Op::Pop]);
    }
    ops.extend([load(owner(Type::Int32)), Op::Field(0), Op::Return]);
    body(&mut module, ops);
    verify(&module).unwrap();
    let result = run(
        &module,
        Limits {
            heap_objects: 3,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.heap.len(), 0);
    assert_eq!(result.heap.statistics().allocated_objects, 21);
    assert_eq!(result.heap.statistics().reclaimed_objects, 21);
}

#[test]
fn readonly_private_and_invalid_stores_are_rejected_without_verification_too() {
    for mode in ["readonly", "private", "index", "type"] {
        let mut module = fixture(Type::Int32);
        match mode {
            "readonly" => module.types[0].static_fields[0].readonly = true,
            "private" => module.types[0].static_fields[0].visibility = Visibility::Private,
            _ => (),
        }
        body(
            &mut module,
            vec![
                if mode == "type" {
                    Op::String("invalid".into())
                } else {
                    Op::Int(1)
                },
                Op::SetStaticField {
                    owner: owner(Type::Int32),
                    index: usize::from(mode == "index"),
                },
                Op::Int(0),
                Op::Return,
            ],
        );
        assert!(verify(&module).is_err(), "{mode}");
        assert!(run(&module, Limits::default()).is_err(), "{mode}");
    }
}

#[test]
fn static_fields_do_not_contribute_to_instance_layout() {
    let mut module = fixture(Type::Named("Item".into()));
    module.types[0].is_reference_type = false;
    module.types[0].fields.push(neoclr::metadata::Field {
        name: "Instance".into(),
        ty: Type::Int32,
        visibility: Visibility::Public,
        deferred: false,
    });
    body(
        &mut module,
        vec![Op::SizeOf(owner(Type::Int32)), Op::ConvertInt32, Op::Return],
    );
    // The static reference does not enlarge the one-Int32 value layout.
    verify(&module).unwrap();
    assert_eq!(
        run(&module, Limits::default()).unwrap().value,
        Value::Int32(4)
    );
}

#[test]
fn inline_array_payloads_in_statics_remain_charged_to_the_invocation() {
    let mut module = fixture(Type::Array(Box::new(Type::Int32)));
    body(
        &mut module,
        vec![
            Op::Int(4),
            Op::Int(0),
            Op::CreateArray(Type::Int32),
            store(owner(Type::Int32)),
            Op::Int(4),
            Op::Int(0),
            Op::CreateArray(Type::Int32),
            Op::Pop,
            Op::Int(0),
            Op::Return,
        ],
    );
    verify(&module).unwrap();
    let fault = run(
        &module,
        Limits {
            array_elements: 6,
            ..Limits::default()
        },
    )
    .unwrap_err();
    assert_eq!(fault.code, neoclr::FaultCode::ArrayLimitExceeded);
}
