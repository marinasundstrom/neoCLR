use neoclr::{Limits, LoadedProgram, Value, frontend};
fn run(source: &str) -> neoclr::Execution {
    let module = frontend::compile(source).unwrap();
    LoadedProgram::new(&module)
        .unwrap()
        .run(Limits::default())
        .unwrap()
}
#[test]
fn synthesized_constructor_uses_field_initializers_for_values_and_heap_objects() {
    let source = "class Counter {\nvar Value: int = default(int)\nfunc Add() -> () { this.Value = this.Value + 1 }\n}\nfunc Main() -> int {\nvar value = Counter()\nlet heap = new Counter()\nvalue.Add()\nheap.Add()\nreturn value.Value + heap.Value\n}";
    assert_eq!(run(source).value, Value::Int32(2));
    let il = frontend::lower_to_il(source).unwrap();
    assert!(il.contains(".method instance byref .ctor()"));
    assert!(il.contains("initobj Int32"));
}
#[test]
fn inherited_initializers_run_after_base_and_before_constructor_body() {
    let source = "import System.Console.*\nfunc Trace(name: string, value: int) -> int { WriteLine(name); return value }\nclass Base {\nvar Value: int = Trace(\"base field\", 10)\ninit() { WriteLine(\"base body\") }\n}\nclass Derived: Base {\nvar Extra: int = Trace(\"derived field\", this.Value + 32)\ninit() { WriteLine(\"derived body\") }\n}\nfunc Main() -> int { return Derived().Extra }";
    let result = run(source);
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(
        result.output,
        ["base field", "base body", "derived field", "derived body"]
    );
}
#[test]
fn abstract_class_views_dispatch_and_explicit_parameters_initialize_required_fields() {
    let source = "abstract class Base {\nvar Value: int\ninit(value: int) { this.Value = value }\nreadonly abstract func Read() -> int\n}\nclass Derived: Base {\ninit(value: int): base(value) {}\nreadonly override func Read() -> int { return this.Value }\n}\nfunc Main() -> int {\nvar derived = Derived(42)\nlet view: Base& = &derived\nreturn view.Read()\n}";
    assert_eq!(run(source).value, Value::Int32(42));
}
#[test]
fn empty_classes_synthesize_and_records_keep_positional_construction() {
    assert_eq!(run("class Empty {}\nclass Child: Empty {}\nrecord Pair(Value: int)\nfunc Main() -> int { let child = Child(); return Pair(42).Value }").value, Value::Int32(42));
}
#[test]
fn invalid_construction_and_field_contracts_fail() {
    for source in [
        "class C { var Value: int }",
        "class C { var Ref: int& = default(int&) }",
        "class C { var Value: int = 0; var Value: int = 1 }",
        "class C { var Value: int = value; init(value: int) {} }",
        "class C { let Value: int = 0 }",
        "class C(Value: int) {}",
        "class C { var Value: int; init() {} }",
        "class Base { init(x: int) {} }; class C: Base {}",
    ] {
        let source = format!("{source}\nfunc Main() -> () {{}}");
        let result = frontend::compile(&source);
        assert!(result.is_err(), "accepted {source}");
    }
}

#[test]
fn sample_uses_class_values_and_managed_base_views() {
    let result = run(include_str!("../examples/source/classes.neo"));
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.output, ["42", "42"]);
}

#[test]
fn nonnullable_reference_fields_require_valid_initialization_even_without_verification() {
    let good = "class Counter { var Value: int = 42 }\nclass Holder { var Target: Counter& = new Counter() }\nfunc Main() -> int { return Holder().Target.Value }";
    assert_eq!(run(good).value, Value::Int32(42));
    let bad =
        "class Holder { var Target: int&; init() {} }\nfunc Main() -> () { let holder = Holder() }";
    assert!(frontend::compile(bad).is_err());
    let module = neoclr::assemble(&frontend::lower_to_il(bad).unwrap()).unwrap();
    let error = LoadedProgram::new(&module)
        .unwrap()
        .run(Limits::default())
        .unwrap_err();
    assert!(error.message.contains("uninitialized"), "{}", error.message);
}

#[test]
fn runtime_checks_reference_field_completion_on_the_executed_return_path() {
    let source = "class Target {}\nclass Holder {\nvar Target: Target&\ninit(target: Target&, ready: bool) { if ready { this.Target = target } }\n}\nfunc Main() -> () { let target = new Target(); let holder = Holder(target, false) }";
    assert!(frontend::compile(source).is_err());
    let module = neoclr::assemble(&frontend::lower_to_il(source).unwrap()).unwrap();
    let error = LoadedProgram::new(&module)
        .unwrap()
        .run(Limits::default())
        .unwrap_err();
    assert!(error.message.contains("uninitialized"));
    let source = source.replace("Holder(target, false)", "Holder(target, true)");
    let module = neoclr::assemble(&frontend::lower_to_il(&source).unwrap()).unwrap();
    LoadedProgram::new(&module)
        .unwrap()
        .run(Limits::default())
        .unwrap();
}
