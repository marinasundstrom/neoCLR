use neoclr::metadata::{Instruction as Op, StaticField, Type, Visibility};
use neoclr::{Limits, Value, assemble, run, verify};

fn fixture(generic: bool) -> neoclr::Module {
    let suffix = if generic { "<T>" } else { "" };
    let mut m = assemble(&format!(".module Initializers\n.entry Main\n.type class Cache{suffix}\n.method private static .cctor() -> noresult\nret\n.end\n.method static Read() -> Int32\nldc.i4 0\nret\n.end\n.end\n.function Main() -> Int32\nldc.i4 0\nret\n.end")).unwrap();
    m.types[0].static_fields.push(StaticField {
        name: "Value".into(),
        ty: Type::Int32,
        visibility: Visibility::Public,
        readonly: true,
    });
    m
}
fn load(owner: &Type) -> Op {
    Op::StaticField {
        owner: owner.clone(),
        index: 0,
    }
}
fn store(owner: &Type) -> Op {
    Op::SetStaticField {
        owner: owner.clone(),
        index: 0,
    }
}
fn body(m: &mut neoclr::Module, name: &str, body: Vec<Op>) {
    m.functions
        .iter_mut()
        .find(|f| f.name == name)
        .unwrap()
        .body = body;
}
fn result(m: &neoclr::Module) -> i32 {
    verify(m).unwrap();
    let Value::Int32(value) = run(m, Limits::default()).unwrap().value else {
        panic!()
    };
    value
}

#[test]
fn initializer_is_lazy_once_only_recursive_and_storage_backed() {
    let mut m = fixture(false);
    let ty = Type::Named("Cache".into());
    // Reading our own field sees its zero default; a second invocation would yield 84.
    body(
        &mut m,
        "Cache..cctor",
        vec![load(&ty), Op::Int(42), Op::Add, store(&ty), Op::Return],
    );
    body(
        &mut m,
        "Main",
        vec![load(&ty), Op::Pop, load(&ty), Op::Return],
    );
    assert_eq!(result(&m), 42);
    assert_eq!(result(&m), 42); // New invocation, fresh storage and initialization state.
    let image = neoclr::metadata_container::write_module(&m).unwrap();
    let restored = neoclr::metadata_container::load_envelope(&image).unwrap();
    assert_eq!(result(&restored), 42);
    body(
        &mut m,
        "Cache..cctor",
        vec![Op::Fault("must stay lazy".into())],
    );
    body(&mut m, "Main", vec![Op::Int(42), Op::Return]);
    assert_eq!(result(&m), 42);
}

#[test]
fn closed_generics_initialize_independently() {
    let mut m = fixture(true);
    let open = Type::Constructed {
        definition: "Cache".into(),
        arguments: vec![Type::TypeParameter(0)],
    };
    let a = Type::Constructed {
        definition: "Cache".into(),
        arguments: vec![Type::Int32],
    };
    let b = Type::Constructed {
        definition: "Cache".into(),
        arguments: vec![Type::String],
    };
    body(
        &mut m,
        "Cache..cctor",
        vec![load(&open), Op::Int(21), Op::Add, store(&open), Op::Return],
    );
    body(
        &mut m,
        "Main",
        vec![load(&a), Op::Pop, load(&a), load(&b), Op::Add, Op::Return],
    );
    assert_eq!(result(&m), 42);
}

#[test]
fn strict_method_trigger_and_relaxed_field_trigger_differ() {
    let mut m = fixture(false);
    body(
        &mut m,
        "Cache..cctor",
        vec![Op::Fault("initializer ran".into())],
    );
    body(&mut m, "Cache.Read", vec![Op::Int(42), Op::Return]);
    body(
        &mut m,
        "Main",
        vec![
            Op::Call(neoclr::assembler::parse_function_ref("Cache::Read()").unwrap()),
            Op::Return,
        ],
    );
    verify(&m).unwrap();
    assert_eq!(
        run(&m, Limits::default()).unwrap_err().code,
        neoclr::FaultCode::UserFault
    );
    m.types[0].before_field_init = true;
    assert_eq!(result(&m), 42);
    body(
        &mut m,
        "Cache.Read",
        vec![load(&Type::Named("Cache".into())), Op::Return],
    );
    assert_eq!(
        run(&m, Limits::default()).unwrap_err().code,
        neoclr::FaultCode::UserFault
    );
}

#[test]
fn ordinary_writes_direct_calls_and_invalid_initializers_are_rejected() {
    let m = fixture(false);
    for invalid in [
        "write",
        "call",
        "entry",
        "public",
        "instance",
        "parameter",
        "result",
    ] {
        let mut m = m.clone();
        match invalid {
            "write" => body(
                &mut m,
                "Main",
                vec![
                    Op::Int(42),
                    store(&Type::Named("Cache".into())),
                    Op::Int(0),
                    Op::Return,
                ],
            ),
            "call" => body(
                &mut m,
                "Main",
                vec![
                    Op::Call(neoclr::assembler::parse_function_ref("Cache::.cctor()").unwrap()),
                    Op::Int(0),
                    Op::Return,
                ],
            ),
            "entry" => m.entry = "Cache..cctor".into(),
            other => {
                let f = &mut m.functions[0];
                match other {
                    "public" => f.visibility = Visibility::Public,
                    "instance" => f.instance = true,
                    "parameter" => f.parameters.push(Type::Int32),
                    "result" => f.no_result = false,
                    _ => unreachable!(),
                }
            }
        }
        assert!(verify(&m).is_err(), "{invalid}");
        assert!(run(&m, Limits::default()).is_err(), "{invalid}");
    }
}

#[test]
fn cross_type_recursion_observes_defaults_without_rerunning_initializers() {
    let mut m = assemble(".module Cycle\n.entry Main\n.type class A\n.method private static .cctor() -> noresult\nret\n.end\n.end\n.type class B\n.method private static .cctor() -> noresult\nret\n.end\n.end\n.function Main() -> Int32\nldc.i4 0\nret\n.end").unwrap();
    for ty in &mut m.types {
        ty.static_fields.push(StaticField {
            name: "Value".into(),
            ty: Type::Int32,
            visibility: Visibility::Public,
            readonly: true,
        });
    }
    let a = Type::Named("A".into());
    let b = Type::Named("B".into());
    body(
        &mut m,
        "A..cctor",
        vec![load(&b), Op::Int(2), Op::Add, store(&a), Op::Return],
    );
    body(
        &mut m,
        "B..cctor",
        vec![load(&a), Op::Int(20), Op::Add, store(&b), Op::Return],
    );
    body(
        &mut m,
        "Main",
        vec![load(&a), load(&b), Op::Add, Op::Return],
    );
    assert_eq!(result(&m), 42);
}

#[test]
fn initializing_one_closed_type_cannot_write_anothers_readonly_field() {
    let mut m = fixture(true);
    let other = Type::Constructed {
        definition: "Cache".into(),
        arguments: vec![Type::Int32],
    };
    body(
        &mut m,
        "Cache..cctor",
        vec![Op::Int(42), store(&other), Op::Return],
    );
    assert!(
        verify(&m)
            .unwrap_err()
            .message
            .contains("readonly static field")
    );
}
