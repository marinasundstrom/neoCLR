use neoclr::{Limits, LoadedProgram, Value, frontend, verify};
fn execute(source: &str) -> Value {
    let module = frontend::compile(source).unwrap();
    verify(&module).unwrap();
    LoadedProgram::new(&module)
        .unwrap()
        .run(Limits::default())
        .unwrap()
        .value
}
#[test]
fn free_static_and_namespace_qualified_functions_share_generic_calls() {
    let source = "func Utility.Identity<T>(value: T) -> T { return value }\nrecord Helpers() {\nstatic func Forward<U>(value: U) -> U { return Utility.Identity(value) }\n}\nfunc Main() -> int { return Helpers.Forward(42) }";
    assert_eq!(execute(source), Value::Int32(42));
}
#[test]
fn generic_typeof_and_reference_arguments_are_transparent() {
    let source = "func Identity<T>(value: T) -> T { return value }\nfunc Same<T>() -> bool { return typeof(T).Equals(typeof(int)) }\nfunc Main() -> int {\nvar value = 42\nlet alias: int& = Identity<int&>(&value)\nif Same<int>() { return alias }\nreturn 0\n}";
    assert_eq!(execute(source), Value::Int32(42));
}
#[test]
fn comparisons_remain_comparisons_and_explicit_init_remains_supported() {
    assert_eq!(
        execute(
            "record Counter(Value: int) { init() { this.Value = 42 } }\nfunc Main() -> int {\nlet c = Counter()\nif c.Value < 50 { return c.Value }\nreturn 0\n}"
        ),
        Value::Int32(42)
    );
}
#[test]
fn unsupported_or_incorrect_generic_calls_fail() {
    for source in [
        "func Id<T>(x: T, y: T) -> T { return x }\nfunc Main() -> int { return Id(42, true) }",
        "func Id<T>(x: T) -> T { return x }\nfunc Main() -> int { return Id<int,string>(42) }",
        "record C() { func Id<T>(x: T) -> T { return x } }\nfunc Main() -> int { return 0 }",
        "func Id<T,T>(x: T) -> T { return x }\nfunc Main() -> int { return 0 }",
        "record C() { static func F() -> int { return this.Value } }\nfunc Main() -> int { return C.F() }",
    ] {
        assert!(frontend::compile(source).is_err(), "{source}");
    }
}

#[test]
fn sample_uses_generic_functions_without_losing_reference_access() {
    assert_eq!(
        execute(include_str!("../examples/source/generic-functions.neo")),
        Value::Int32(42)
    );
}

#[test]
fn generic_method_reflection_reports_its_current_limit() {
    let module = frontend::compile("record Helpers() { static func Id<T>(x: T) -> T { return x } }\nfunc Main() -> () { typeof(Helpers).GetMethods() }").unwrap();
    let error = LoadedProgram::new(&module)
        .unwrap()
        .run(Limits::default())
        .unwrap_err();
    assert!(
        error
            .message
            .contains("generic method definition reflection"),
        "{}",
        error.message
    );
}

#[test]
fn inference_respects_value_and_reference_intent_and_emits_arguments_once() {
    let source = "func Id<T>(x: T) -> T { return x }\nfunc Read<T>(x: T&) -> T { return x }\nfunc Next(x: int&) -> int { x = x + 1; return x }\nfunc Main() -> int {\nvar x = 40\nlet alias = Id(&x)\nlet copy: int = Id(alias)\nlet result = Id(Next(alias))\nreturn Read(alias) + result - copy\n}";
    assert_eq!(execute(source), Value::Int32(42));
}

#[test]
fn structural_array_and_readonly_reference_inference_work() {
    let source = "func First<T>(items: T[]) -> T { return items[0] }\nfunc Read<T>(readonly item: T&) -> T { return item }\nfunc Main() -> int {\nlet value = 42\nlet items = [Read(&value), 0]\nreturn First(items)\n}";
    assert_eq!(execute(source), Value::Int32(42));
}

#[test]
fn inference_requires_evidence_and_has_no_return_target_typing() {
    let error = frontend::compile(
        "func Kind<T>() -> System.Type { return typeof(T) }\nfunc Main() -> () { Kind() }",
    )
    .unwrap_err();
    assert!(error.message.contains("cannot infer"));
}

#[test]
fn nested_generic_calls_do_not_duplicate_inference_exponentially() {
    let mut expression = "42".to_owned();
    for _ in 0..24 {
        expression = format!("Id({expression})");
    }
    assert_eq!(
        execute(&format!(
            "func Id<T>(x: T) -> T {{ return x }}\nfunc Main() -> int {{ return {expression} }}"
        )),
        Value::Int32(42)
    );
}

#[test]
fn inference_preserves_an_existing_managed_reference_binding() {
    let source = "func Id<T>(x: T) -> T { return x }\nfunc Main() -> int {\nvar x = 1\nlet original = &x\nlet alias = Id(original)\nalias = 42\nreturn x\n}";
    assert_eq!(execute(source), Value::Int32(42));
    assert!(
        frontend::lower_to_il(source)
            .unwrap()
            .contains("call Id<Int32&>(Int32&)")
    );
}
