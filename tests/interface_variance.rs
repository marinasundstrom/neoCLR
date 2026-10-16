use neoclr::{Limits, LoadedProgram, Value, assemble};

fn image(declarations: &str) -> neoclr::Module {
    assemble(&format!(".module Variance\n.entry Main\n{declarations}\n.function Main() -> Int32\nldc.i4 42\nret\n.end")).unwrap()
}

#[test]
fn variance_metadata_roundtrip_and_nested_polarity() {
    let module = image(
        ".interface Read<T>\n.variance out\n.method instance Get() -> T\n.end\n.end\n.interface Write<T>\n.variance in\n.method instance Put(T) -> Int32\n.end\n.end\n.interface Nested<T>\n.variance out\n.implements Read<T>\n.method instance Supply(Write<T>) -> Read<T>\n.end\n.end",
    );
    let json = serde_json::to_string(&module).unwrap();
    assert!(json.contains("covariant"));
    let restored = serde_json::from_str(&json).unwrap();
    let p = LoadedProgram::new(&restored).unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}

#[test]
fn malformed_and_unsafe_declarations_are_rejected() {
    for declaration in [
        ".interface Bad<T>\n.variance out in\n.end",
        ".type class Bad<T>\n.variance out\n.end",
        ".interface Bad<T>\n.variance out\n.method instance Put(T) -> Int32\n.end\n.end",
        ".interface Bad<T>\n.variance in\n.method instance Get() -> T\n.end\n.end",
        ".interface Bad<T>\n.variance out\n.method instance Get() -> arrayref<T>\n.end\n.end",
        ".interface Bad<T>\n.variance out\n.method instance Get() -> T&\n.end\n.end",
        ".type Result<T>\n.end\n.interface Bad<T>\n.variance out\n.method instance Get() -> Result<T>\n.end\n.end",
        ".interface Invariant<T>\n.end\n.interface Bad<T>\n.variance out\n.implements Invariant<T>\n.end",
        ".interface Identity<T>\n.end\n.interface Bad<T>\n.variance out\n.implements Identity<Self>\n.end",
        ".interface Sink<T>\n.variance in\n.end\n.interface Bad<T>\n.variance out\n.method instance Get() -> Sink<T>\n.end\n.end",
    ] {
        let module = assemble(&format!(
            ".module Variance\n.entry Main\n{declaration}\n.function Main() -> Int32\nldc.i4 42\nret\n.end"
        ));
        assert!(
            module
                .and_then(|m| LoadedProgram::new(&m))
                .and_then(|p| p.verify())
                .is_err(),
            "{declaration}"
        );
    }
    let mut json = serde_json::to_value(image(".interface Read<T>\n.end")).unwrap();
    json["types"][0]["generic_variance"] = serde_json::json!(["unknown"]);
    assert!(serde_json::from_value::<neoclr::Module>(json).is_err());
}

#[test]
fn omitted_variance_remains_invariant_metadata() {
    let module = image(".interface Read<T>\n.method instance Get() -> T\n.end\n.end");
    assert!(module.types[0].generic_variance.is_empty());
    assert!(
        !serde_json::to_string(&module)
            .unwrap()
            .contains("generic_variance")
    );
    LoadedProgram::new(&module).unwrap().verify().unwrap();
}

const REFERENCES: &str = r#"
.type class System.Object
.end
.type class Base
.extends System.Object
.end
.type class Derived
.extends Base
.end
.interface Producer<T>
.variance out
.method instance Get() -> T
.end
.end
.interface Consumer<T>
.variance in
.method instance Accept(T) -> Int32
.end
.end
.type class Source
.implements Producer<Derived>
.field Item Derived
.method instance Get() -> Derived
ldarg this
ldfld 0
ret
.end
.end
.type class Sink
.implements Consumer<Base>
.method instance Accept(Base value) -> Int32
newobj Base
starg value
ldc.i4 42
ret
.end
.end
"#;

#[test]
fn reference_covariance_and_contravariance_dispatch_without_copying_receiver() {
    let text = include_str!("../docs/validation/interface-variance/Main.neoil");
    let module = assemble(text).unwrap();
    let p = LoadedProgram::new(&module).unwrap();
    p.verify().unwrap();
    let graph = p
        .analyze_reachability(
            &[neoclr::assembler::parse_function_ref("Main()").unwrap()],
            100,
        )
        .unwrap();
    assert!(
        graph
            .functions
            .iter()
            .any(|f| f.target.name == "Source.Get")
    );
    assert!(
        graph
            .functions
            .iter()
            .any(|f| f.target.name == "Sink.Accept")
    );
    let result = p.run(Limits::default()).unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.heap.statistics().allocated_objects, 4);
}

#[test]
fn reversed_and_value_argument_variance_are_rejected() {
    for (source, target) in [
        ("Producer<Base>", "Producer<Derived>"),
        ("Consumer<Derived>", "Consumer<Base>"),
        ("Producer<Int32>", "Producer<System.Object>"),
        ("Producer<Int32>", "Producer<Int64>"),
        ("Producer<System.Object>", "Producer<String>"),
        ("Consumer<String>", "Consumer<System.Object>"),
    ] {
        let text = format!(
            ".module Variance\n.entry Main\n{REFERENCES}\n.function Convert({source} value) -> {target}\nldarg value\nret\n.end\n.function Main() -> Int32\nldc.i4 42\nret\n.end"
        );
        assert!(
            assemble(&text)
                .and_then(|m| LoadedProgram::new(&m))
                .and_then(|p| p.verify())
                .is_err(),
            "{source} -> {target}"
        );
    }
}

#[test]
fn exact_view_wins_and_ambiguous_widening_fails() {
    for exact in [false, true] {
        let extra = if exact {
            ".implements Marker<Base>"
        } else {
            ""
        };
        let module = assemble(&format!(
            r#".module Variance
.entry Main
{REFERENCES}
.type class Other
.extends Base
.end
.interface Marker<T>
.variance out
.end
.type class Both
.implements Marker<Derived>
.implements Marker<Other>
{extra}
.end
.function Main() -> Int32
.local Marker<Base> view
newobj Both
castclass Marker<Base>
stloc view
ldc.i4 42
ret
.end"#
        ))
        .unwrap();
        let result = LoadedProgram::new(&module).unwrap().run(Limits::default());
        if exact {
            assert_eq!(result.unwrap().value, Value::Int32(42));
        } else {
            assert!(result.unwrap_err().message.contains("ambiguous"));
        }
    }
}

#[test]
fn matching_methods_do_not_establish_nominal_conformance() {
    let module = assemble(&format!(
        r#".module Variance
.entry Main
{REFERENCES}
.type class Pretender
.method instance Get() -> Derived
newobj Derived
ret
.end
.end
.function Main() -> Int32
.local Producer<Base> view
newobj Pretender
castclass Producer<Base>
stloc view
ldc.i4 42
ret
.end"#
    ))
    .unwrap();
    assert!(
        LoadedProgram::new(&module)
            .unwrap()
            .run(Limits::default())
            .is_err()
    );
}

#[test]
fn closed_generic_execution_does_not_advertise_an_incomplete_dispatch_graph() {
    let module = assemble(&format!(
        r#".module Variance
.entry Main
{REFERENCES}
.type class Box<T>
.implements Producer<T>
.field Item T
.method instance Get() -> T
ldarg this
ldfld 0
ret
.end
.end
.function Main() -> Int32
.local Producer<Base> view
newobj Derived
newobj Box<Derived>
castclass Producer<Base>
stloc view
ldloc view
callvirt instance Producer<Base>::Get()
pop
ldc.i4 42
ret
.end"#
    ))
    .unwrap();
    let p = LoadedProgram::new(&module).unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    let error = p
        .analyze_reachability(
            &[neoclr::assembler::parse_function_ref("Main()").unwrap()],
            100,
        )
        .unwrap_err();
    assert!(
        error
            .message
            .contains("requires nongeneric implementing types")
    );
}

#[test]
fn already_boxed_receiver_widens_without_allocating_another_box() {
    let module = assemble(&format!(
        r#".module Variance
.entry Main
{REFERENCES}
.type ValueSource
.implements Producer<Derived>
.field Item Derived
.method instance byref Get() -> Derived
ldarg this
ldfld 0
ret
.end
.end
.function Main() -> Int32
.local Producer<Derived> narrow
.local Producer<Base> wide
newobj Derived
newobj ValueSource
box ValueSource
castclass Producer<Derived>
stloc narrow
ldloc narrow
stloc wide
ldloc wide
ldloc narrow
ref.eq
brfalse failed
ldloc wide
callvirt instance Producer<Base>::Get()
pop
ldc.i4 42
ret
failed:
ldc.i4 1
ret
.end"#
    ))
    .unwrap();
    let p = LoadedProgram::new(&module).unwrap();
    p.verify().unwrap();
    let graph = p
        .analyze_reachability(
            &[neoclr::assembler::parse_function_ref("Main()").unwrap()],
            100,
        )
        .unwrap();
    assert!(
        graph
            .functions
            .iter()
            .any(|f| f.target.name == "ValueSource.Get")
    );
    let result = p.run(Limits::default()).unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.heap.statistics().allocated_objects, 2);
}

#[test]
fn string_variant_calls_materialize_identity_preserving_views() {
    let module = assemble(
        r#"
.module StringVariance
.entry Main
.type class System.Object
.end
.interface Producer<T>
.variance out
.method instance Get() -> T
.end
.end
.interface Consumer<T>
.variance in
.method instance Accept(T value) -> Int32
.end
.end
.type class Source
.implements Producer<String>
.field Text String
.method instance Get() -> String
ldarg this
ldfld 0
ret
.end
.end
.type class Sink
.implements Consumer<System.Object>
.field Expected String
.method instance Accept(System.Object value) -> Int32
ldarg value
ldarg this
ldfld 0
ref.eq
brfalse Failed
ldarga value
initobj System.Object
ldc.i4 42
ret
Failed:
ldc.i4 1
ret
.end
.end
.function Main() -> Int32
.local String text
.local Producer<System.Object> producer
.local System.Object first
ldstr "hé😀"
stloc text
ldloc text
newobj Source
castclass Producer<System.Object>
stloc producer
ldloc producer
callvirt instance Producer<System.Object>::Get()
stloc first
ldloc first
ldloc text
ref.eq
brfalse Failed
ldloc first
ldloc producer
callvirt instance Producer<System.Object>::Get()
ref.eq
brfalse Failed
ldloc text
newobj Sink
castclass Consumer<String>
ldloc text
callvirt instance Consumer<String>::Accept(String)
ret
Failed:
ldc.i4 2
ret
.end
"#,
    )
    .unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    let result = program.run(Limits::default()).unwrap();
    assert_eq!(result.value, Value::Int32(42));
    // Two receivers and three ordinary String view handles; no text owner copy.
    assert_eq!(result.heap.statistics().allocated_objects, 5);
    let fault = program
        .run(Limits {
            heap_objects: 1,
            ..Limits::default()
        })
        .unwrap_err();
    assert_eq!(fault.code, neoclr::FaultCode::HeapLimitExceeded);
    let mut failing = module.clone();
    failing
        .functions
        .iter_mut()
        .find(|f| f.name == "Source.Get")
        .unwrap()
        .body = vec![neoclr::metadata::Instruction::Fault(
        "string producer failed".into(),
    )];
    let failing = LoadedProgram::new(&failing).unwrap();
    failing.verify().unwrap();
    assert!(
        failing
            .run(Limits::default())
            .unwrap_err()
            .to_string()
            .contains("string producer failed")
    );
}
