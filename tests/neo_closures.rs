use neoclr::{Limits, LoadedProgram, Value};

fn run(source: &str) -> Value {
    let module = neoclr::frontend::compile(source).unwrap();
    LoadedProgram::new(&module)
        .unwrap()
        .run(Limits::default())
        .unwrap()
        .value
}

#[test]
fn noncapturing_and_contextual_lambdas() {
    assert_eq!(
        run(r#"
func Apply(f: fn<int, int>) -> int { return f(41) }
func Main() -> int {
    let noop: fn<void> = () => {}
    noop()
    return Apply((x: int) => x + 1)
}
"#),
        Value::Int32(42)
    );
}

#[test]
fn captures_share_mutable_storage() {
    assert_eq!(
        run(r#"
func Main() -> int {
    var value = 1
    let before = &value
    let add: fn<int, void> = amount => { value = value + amount }
    let read: fn<int> = () => value
    value = 40
    add(2)
    return read() + before - value
}
"#),
        Value::Int32(42)
    );
}

#[test]
fn returned_closure_and_nested_parameter_capture() {
    assert_eq!(
        run(r#"
func Make(start: int) -> fn<int, fn<int>> {
    var value = start
    return amount => { return () => { value = value + amount; return value } }
}
func Main() -> int {
    let factory = Make(40)
    let increment = factory(1)
    increment()
    return increment()
}
"#),
        Value::Int32(42)
    );
}

#[test]
fn generic_capture() {
    assert_eq!(
        run(r#"
func Keep<T>(value: T) -> fn<T> { return () => value }
func Main() -> int { return Keep<int>(42)() }
"#),
        Value::Int32(42)
    );
}

#[test]
fn loop_bindings_are_fresh_and_lambdas_work_in_fields() {
    assert_eq!(
        run(r#"
class Holder { var Callback: fn<int> = () => 0 }
func Main() -> int {
    let callbacks = new Holder[3] { Holder(), Holder(), Holder() }
    for i in 0..<3 { callbacks[i].Callback = () => i }
    return callbacks[0].Callback() * 100 + callbacks[1].Callback() * 10 + callbacks[2].Callback()
}
"#),
        Value::Int32(12)
    );
}

#[test]
fn explicit_construction_and_void_expression_body() {
    assert_eq!(
        run(r#"
func Ignore(value: int) -> () {}
func Main() -> int {
    let callback = fn<int,Void>(x => Ignore(x))
    callback(42)
    return 42
}
"#),
        Value::Int32(42)
    );
}

#[test]
fn captured_heap_reference_and_this() {
    assert_eq!(
        run(r#"
class Counter {
    var Value: int = 40
    func Make() -> fn<int> { return () => { this.Value = this.Value + 1; return this.Value } }
}
func Keep(value: Counter&) -> fn<int> { return () => value.Value }
func Main() -> int {
    let counter = new Counter()
    let next = counter.Make()
    let read = Keep(counter)
    next()
    next()
    return read()
}
"#),
        Value::Int32(42)
    );
}

#[test]
fn frame_reference_capture_faults_at_runtime() {
    let m = neoclr::frontend::compile(
        r#"
func Keep(value: int&) -> fn<int> { return () => value }
func Main() -> int { var value = 42; let callback = Keep(&value); return callback() }
"#,
    )
    .unwrap();
    let error = LoadedProgram::new(&m)
        .unwrap()
        .run(Limits::default())
        .unwrap_err();
    assert!(error.message.contains("frame"), "{error}");
}

#[test]
fn invalid_captures_and_lambda_contracts_are_rejected() {
    for source in [
        "func Main() -> int { let x = 1; let f: fn<void> = () => { x = 2 }; return x }",
        "func Main() -> int { var x: int; let f: fn<int> = () => x; return f() }",
        "func Keep(out x: int&) -> fn<int> { x = 42; return () => x }; func Main() -> int { return 0 }",
        "func Main() -> int { let f = x => x; return 0 }",
        "func Main() -> int { let f: fn<int> = x => x; return 0 }",
        "func Main() -> int { let f: fn<int,int> = (x: bool) => 1; return 0 }",
        "func Main() -> int { let f: fn<int,int,int> = (x,x) => x; return 0 }",
        "class C { var Callback: fn<int>; init() { this.Callback = () => this.Read() }; func Read() -> int { return 1 } }; func Main() -> int { return 0 }",
    ] {
        assert!(neoclr::frontend::compile(source).is_err(), "{source}");
    }
}

#[test]
fn closure_cells_survive_collection_and_artifact_roundtrip() {
    let m = neoclr::frontend::compile(
        r#"
func Keep(value: int) -> fn<int> { return () => value }
func Main() -> int {
    let read = Keep(42)
    for i in 0..<100 { let discarded = Keep(i) }
    return read()
}
"#,
    )
    .unwrap();
    let m = neoclr::load(&serde_json::to_string(&m).unwrap()).unwrap();
    let result = LoadedProgram::new(&m)
        .unwrap()
        .run(Limits {
            heap_objects: 10,
            ..Limits::default()
        })
        .unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert!(result.heap.statistics().collections > 0);
}

#[test]
fn lambdas_inherit_out_and_readonly_parameter_contracts() {
    assert_eq!(
        run(r#"
func Main() -> int {
    let set: fn<out int&,Void> = value => { value = 42 }
    let read: fn<readonly int&,int> = value => value
    var value: int
    set(out value)
    return read(&value)
}
"#),
        Value::Int32(42)
    );
}

#[test]
fn shadowed_names_and_generic_noncapturing_helper() {
    assert_eq!(
        run(r#"
func Identity<T>() -> fn<T,T> { return x => x }
func Main() -> int {
    var x = 40
    let f: fn<int,int> = x => x + 1
    let g: fn<int> = () => { let x = 42; return x }
    return Identity<int>()(f(x)) + g() - 41
}
"#),
        Value::Int32(42)
    );
}

#[test]
fn closure_cycles_are_collectible() {
    let m = neoclr::frontend::compile(
        r#"
class Holder { var Callback: fn<int> = () => 0 }
func Main() -> int {
    for i in 0..<100 {
        let holder = new Holder()
        holder.Callback = () => holder.Callback()
    }
    return 42
}
"#,
    )
    .unwrap();
    let result = LoadedProgram::new(&m)
        .unwrap()
        .run(Limits {
            heap_objects: 12,
            ..Limits::default()
        })
        .unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert!(result.heap.statistics().collections > 0);
}

#[test]
fn composed_readonly_func_and_annotated_readonly_parameters() {
    assert_eq!(
        run(r#"
func Main() -> int {
    var value = 42
    let read: fn<readonly int&, int> = value => value
    let annotated: fn<readonly int&,int> = (value: readonly int&) => value
    return read(&value) + annotated(&value) - 42
}
"#),
        Value::Int32(42)
    );
}

#[test]
fn match_payload_capture_outlives_its_arm() {
    assert_eq!(
        run(r#"
func Parse(text: string) -> fn<int> {
    return Int32.Parse(text) match {
        Ok(let value) => () => value,
        Error(_) => () => 0
    }
}
func Main() -> int { return Parse("42")() }
"#),
        Value::Int32(42)
    );
}

#[test]
fn closure_sample_runs() {
    assert_eq!(
        run(include_str!("../examples/source/closures.neo")),
        Value::Int32(42)
    );
}
