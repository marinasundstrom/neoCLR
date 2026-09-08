use neoclr::{Limits, LoadedProgram, Value, assemble, frontend};

#[test]
fn discovery_preserves_concrete_interface_type_without_retaining_heap_targets() {
    let source = r#"
interface Counter { func Read() -> int }
record Concrete(Age: int): Counter { func Read() -> int { return this.Age } }
func Discover() -> System.Type {
    let owner = new Concrete(1)
    let view: Counter& = owner
    return view.GetType()
}
func Main() -> string {
    var local = Concrete(2)
    let view: Counter& = &local
    if !view.GetType().Equals(typeof(Concrete)) { return "wrong frame type" }
    let age = &local.Age
    if !age.GetType().Equals(typeof(int)) { return "wrong interior type" }
    return Discover().Name
}
"#;
    let module = frontend::compile(source).unwrap();
    let loaded = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    let result = LoadedProgram::new(&loaded)
        .unwrap()
        .run(Limits {
            heap_objects: 1,
            ..Limits::default()
        })
        .unwrap();
    assert_eq!(result.value, Value::String("Concrete".into()));
    assert_eq!(result.heap.statistics().live_objects, 0);
}

#[test]
fn discovery_rejects_values_pointers_and_uninitialized_targets() {
    for body in ["ldc.i4 1", "ptr.null Int32", "ldloca value"] {
        let source = format!(
            ".module App\n.entry Main\n.function Main() -> System.RuntimeTypeHandle\n.local Int32 value\n{body}\nref.type\nret\n.end"
        );
        let module = assemble(&source).unwrap();
        let program = LoadedProgram::new(&module).unwrap();
        assert!(program.verify().is_err());
        assert!(program.run(Limits::default()).is_err());
    }
}

#[test]
fn declared_get_type_methods_keep_normal_dispatch() {
    let source = "record Custom() { func GetType() -> int { return 7 } }\nfunc Main() -> int { let value = new Custom()\nreturn value.GetType() }";
    let module = frontend::compile(source).unwrap();
    assert_eq!(
        LoadedProgram::new(&module)
            .unwrap()
            .run(Limits::default())
            .unwrap()
            .value,
        Value::Int32(7)
    );
}
