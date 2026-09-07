use neoclr::{Limits, LoadedProgram, Value, frontend};

fn run(source: &str) -> neoclr::Execution {
    LoadedProgram::new(&frontend::compile(source).unwrap())
        .unwrap()
        .run(Limits::default())
        .unwrap()
}

#[test]
fn scalar_references_are_read_and_written_in_value_contexts() {
    let source = "func Echo(value: int) -> int { return value }\nfunc Read(value: int&) -> int { return value }\nfunc Main() -> int { var x = 40; let age = &x; age = age + 2; let copy: int = age; Console.WriteLine(age); Console.WriteLine(Echo(age)); Console.WriteLine(age.ToString()); return Read(age) + copy }";
    let result = run(source);
    assert_eq!(result.output, ["42", "42", "42"]);
    assert_eq!(result.value, Value::Int32(84));
    assert_eq!(run("func Main() -> int { var x = 1; let r = &x; var y = 7; let other = &y; r = other; other = 9; return x }").value, Value::Int32(7));
}

#[test]
fn reference_parameters_forward_and_rebinding_is_explicit() {
    let source = "func Add(value: int&) -> () { value = value + 1 }\nfunc Forward(value: int&) -> int& { Add(&value); return &value }\nfunc Main() -> int { var a = 1; var b = 40; var reference = &a; reference = &b; let alias = Forward(reference); alias = alias + 1; return a + b }";
    assert_eq!(run(source).value, Value::Int32(43));
    assert!(
        frontend::compile("func Main() -> () { var a = 1; var b = 2; let r = &a; r = &b }")
            .is_err()
    );
    assert!(frontend::compile("func Main() -> () { var a = 1; let r = &a; r = true }").is_err());
}

#[test]
fn record_copies_field_references_and_reference_results_need_no_star() {
    let source = "record Counter(Age: int)\nrecord Holder(Age: int&)\nfunc Read(counter: Counter) -> int { return counter.Age }\nfunc Age(counter: Counter&) -> int& { return &counter.Age }\nfunc Main() -> int { let shared = new Counter(1); let holder = Holder(&shared.Age); holder.Age = 40; let alias = &holder.Age; alias = alias + 1; Age(shared) = 42; let copy: Counter = shared; shared = Counter(43); return Read(shared) + copy.Age }";
    assert_eq!(run(source).value, Value::Int32(85));
    assert_eq!(run("record Counter(Age: int)\nfunc Main() -> int { let shared = new Counter(1); let holder = new Counter(40); shared = holder; holder.Age = 42; return shared.Age }").value, Value::Int32(40));
}

#[test]
fn conditions_bounds_and_match_arms_apply_automatic_reads() {
    let source = "func Main() -> int { var flag = true; let condition = &flag; var end = 3; let bound = &end; var total = 0; if condition && !false { for i in 0..<bound { total = total + i } }; condition = false; while condition { total = 100 }; let r = &total; let copy: int = Int32.Parse(\"42\") match { Ok(_) => r, Error(_) => 0 }; return (Int32.Parse(\"42\") match { Ok(_) => r, Error(_) => 0 }) + copy }";
    assert_eq!(run(source).value, Value::Int32(6));
}

#[test]
fn lifetime_checks_and_pointer_boundary_still_apply() {
    for source in [
        "func Main() -> int& { var x = 42; let r = &x; return &r }",
        "func Main() -> int { var x = 42; let r = &x; return *r }",
        "func Main() -> int* { return 0 }",
    ] {
        match frontend::compile(source) {
            Err(_) => (),
            Ok(module) => assert!(
                LoadedProgram::new(&module)
                    .unwrap()
                    .run(Limits::default())
                    .is_err()
            ),
        }
    }
    // A declared value return copies the referent and does not leak the address.
    assert_eq!(
        run("func Main() -> int { var x = 42; let r = &x; return r }").value,
        Value::Int32(42)
    );
}
