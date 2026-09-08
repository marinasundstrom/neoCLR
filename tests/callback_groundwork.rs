//! Existing interface/constructor primitives used as delegate design evidence.
//! These tests do not claim that guest delegates have been implemented.
use neoclr::{Limits, LoadedProgram, Value, frontend};

fn program(source: &str) -> LoadedProgram {
    LoadedProgram::new(&frontend::compile(source).unwrap()).unwrap()
}

#[test]
fn callback_adapters_preserve_frame_identity_and_trace_retained_heap_targets() {
    let module =
        frontend::compile(include_str!("../examples/source/callback-groundwork.neo")).unwrap();
    let module = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    let p = LoadedProgram::new(&module).unwrap();
    let result = p
        .run(Limits {
            heap_objects: 4,
            ..Limits::default()
        })
        .unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.output, ["42", "42", "42"]);
    assert!(result.heap.statistics().collections > 1);
    assert!(result.heap.statistics().peak_objects <= 4);
    let graph = p
        .analyze_reachability(
            &[neoclr::assembler::parse_function_ref("Main()").unwrap()],
            100,
        )
        .unwrap();
    for name in [
        "Increment",
        "Counter.Invoke",
        "BoundCounter.Invoke",
        "StaticAdapter.Invoke",
    ] {
        assert!(
            graph.functions.iter().any(|f| f.target.name == name),
            "{name}"
        );
    }
}

#[test]
fn storing_a_frame_target_in_an_adapter_is_a_runtime_fault() {
    let source = "class Counter { var Value: int = 1 }\nclass Bound { var Target: Counter&; init(target: Counter&) { this.Target = target } }\nfunc Main() -> () { var local = Counter(); let bound = Bound(&local) }";
    let error = program(source).run(Limits::default()).unwrap_err();
    assert!(
        error
            .message
            .contains("frame-backed references cannot escape into stored values"),
        "{}",
        error.message
    );
}

#[test]
fn explicit_value_adapter_copies_while_direct_frame_reference_shares() {
    let source = "class Counter { var Value: int; init(value: int) { this.Value = value }; func Next() -> int { this.Value = this.Value + 1; return this.Value } }\nclass Copy { var Target: Counter; init(target: Counter) { this.Target = target }; func Invoke() -> int { return this.Target.Next() } }\nfunc Main() -> int { var original = Counter(1); var copy = Copy(original); original.Value = 40; let next = copy.Invoke(); if original.Value == 40 { return next }; return 0 }";
    assert_eq!(
        program(source).run(Limits::default()).unwrap().value,
        Value::Int32(2)
    );
}

#[test]
fn frame_interface_callback_cannot_return_its_dead_owner() {
    let source = "interface Callback { func Invoke() -> int }\nclass Counter: Callback { var Value: int = 42; func Invoke() -> int { return this.Value } }\nfunc Escape() -> Callback& { var local = Counter(); return &local }\nfunc Main() -> int { let callback = Escape(); return callback.Invoke() }";
    let error = program(source).run(Limits::default()).unwrap_err();
    assert!(error.message.contains("current frame"), "{}", error.message);
}
