use neoclr::{Fault, Limits, LoadedProgram, Value, frontend};

const COUNTER: &str = "interface Counter { func Add(amount: int) -> (); func Read() -> int }\nrecord SimpleCounter(Value: int): Counter { func Add(amount: int) -> () { this.Value = this.Value + amount }; func Read() -> int { return this.Value } }\n";
fn program(source: &str) -> Result<LoadedProgram, Fault> {
    LoadedProgram::new(&frontend::compile(source)?)
}
fn run(source: &str) -> neoclr::Execution {
    program(source).unwrap().run(Limits::default()).unwrap()
}

#[test]
fn example_dispatches_to_original_frame_and_heap_storage_without_boxes() {
    let source = include_str!("../examples/source/interfaces.neo");
    let il = frontend::lower_to_il(source).unwrap();
    assert!(il.contains(".interface Counter"));
    assert!(il.contains(".implements Counter"));
    assert!(il.contains(".method instance byref Add(Int32)"));
    assert!(il.contains("interface.borrow Counter"));
    assert!(il.contains("callvirt instance Counter::Read()"));
    let result = run(source);
    assert_eq!(result.output, ["3", "3", "42"]);
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.heap.statistics().allocated_objects, 1);
    assert_eq!(result.heap.reclaimed_objects(), 1);
}

#[test]
fn interface_reference_alone_roots_heap_receiver_under_collection_pressure() {
    let p = program(&format!("{COUNTER}\nfunc Make() -> Counter& {{ return new SimpleCounter(40) as Counter& }}\nfunc Discard() -> () {{ let temporary = new SimpleCounter(0) }}\nfunc Main() -> int {{ let view = Make(); Discard(); Discard(); Discard(); view.Add(2); return view.Read() }}")).unwrap();
    let result = p
        .run(Limits {
            heap_objects: 2,
            ..Limits::default()
        })
        .unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.heap.statistics().allocated_objects, 4);
    assert_eq!(result.heap.reclaimed_objects(), 4);
}

#[test]
fn concrete_methods_and_interface_methods_share_reference_arguments_and_returns() {
    let result = run(
        "interface CellAccess { func Address() -> int&; func Fill(value: int&) -> () }\nrecord Cell(Value: int): CellAccess { func Address() -> int& { return &this.Value }; func Fill(value: int&) -> () { value = this.Value } }\nfunc Main() -> int { var cell = Cell(40); let direct = cell.Address(); direct = 42; let view = &cell as CellAccess&; var result = 0; view.Fill(&result); let indirect = view.Address(); indirect = 7; return result + cell.Value }",
    );
    assert_eq!(result.value, Value::Int32(49));
    assert_eq!(result.heap.statistics().allocated_objects, 0);
}

#[test]
fn different_implementations_and_multiple_contracts_dispatch_by_exact_type() {
    let result = run(
        "interface Readable { func Read() -> int }\ninterface Incrementable { func Add(amount: int) -> () }\nrecord A(Value: int): Readable, Incrementable { func Read() -> int { return this.Value }; func Add(amount: int) -> () { this.Value = this.Value + amount } }\nrecord B(Value: int): Readable { func Read() -> int { return this.Value * 2 } }\nfunc Read(value: Readable&) -> int { return value.Read() }\nfunc Main() -> int { var a = A(10); var b = B(15); let add = &a as Incrementable&; add.Add(2); return Read(&a as Readable&) + Read(&b as Readable&) }",
    );
    assert_eq!(result.value, Value::Int32(42));
}

#[test]
fn projecting_an_existing_view_preserves_reference_identity() {
    let result = run(&format!(
        "{COUNTER}\nfunc Main() -> int {{ var owner = SimpleCounter(40); let first = &owner as Counter&; let second = first as Counter&; second.Add(2); return first.Read() }}"
    ));
    assert_eq!(result.value, Value::Int32(42));
}

#[test]
fn returning_a_current_frame_interface_view_faults_without_verification() {
    let source = format!(
        "{COUNTER}\nfunc Main() -> Counter& {{ var local = SimpleCounter(1); return &local as Counter& }}"
    );
    let module = neoclr::assemble(&frontend::lower_to_il(&source).unwrap()).unwrap();
    let p = LoadedProgram::new(&module).unwrap();
    assert!(p.run(Limits::default()).is_err());
}

#[test]
fn invalid_projection_and_dispatch_contracts_are_rejected() {
    for main in [
        "func Main() -> () { var c = SimpleCounter(1); let view = c as Counter& }",
        "func Main() -> () { var c = SimpleCounter(1); let view = &c as Counter }",
        "func Main() -> () { var c = SimpleCounter(1); let view = &c as int& }",
        "func Main() -> () { let c = SimpleCounter(1); let view = &c as Counter& }",
        "func Main() -> () { let c = new SimpleCounter(1); let view = c as Counter&; view.Add(true) }",
        "func Main() -> () { let c = new SimpleCounter(1); let view = c as Counter&; view.Add() }",
        "func Main() -> () { let c = new SimpleCounter(1); let view = c as Counter&; view.Missing() }",
        "func Main() -> () { if true { var c = SimpleCounter(1); let view = &c as Counter& } }",
        "func Main() -> () { let c = SimpleCounter(1); c.Add(2) }",
    ] {
        assert!(
            program(&format!("{COUNTER}\n{main}")).is_err(),
            "accepted {main}"
        );
    }
    for source in [
        "interface Readable { func Read() -> int }; record Cell(Value: int): Readable; func Main() -> () {}",
        "interface Readable { func Read() -> int }; record Cell(Value: int): Readable { func Read() -> bool { return true } }; func Main() -> () {}",
        "interface Readable { func Read() -> int }; record Cell(Value: int) { func Read() -> int { return this.Value } }; func Main() -> () { var c = Cell(1); let i = &c as Readable& }",
        "interface Readable { func Read() -> int }; interface AlternateReadable { func Read() -> int }; record Cell(Value: int): Readable { func Read() -> int { return this.Value } }; func Use(j: AlternateReadable&) -> int { return j.Read() }; func Main() -> int { var c = Cell(1); return Use(&c as Readable&) }",
        "interface Readable { func Read() -> int { return 1 } }; func Main() -> () {}",
        "interface Readable { func Read() -> int; func Read() -> int }; func Main() -> () {}",
        "record Cell(Value: int) { func Read(this: int) -> int { return this } }; func Main() -> () {}",
        "record Cell(Value: int) { func Read() -> int { let this = 1; return this } }; func Main() -> () {}",
    ] {
        assert!(program(source).is_err(), "accepted {source}");
    }
}

#[test]
fn cli_assembles_and_runs_interface_example() {
    let path =
        std::env::temp_dir().join(format!("neoclr-interfaces-{}.neo.json", std::process::id()));
    let build = std::process::Command::new(env!("CARGO_BIN_EXE_neoclr"))
        .args(["assemble", "examples/source/interfaces.neo"])
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let execution = std::process::Command::new(env!("CARGO_BIN_EXE_neoclr"))
        .arg("run")
        .arg(&path)
        .output()
        .unwrap();
    std::fs::remove_file(path).unwrap();
    assert!(
        execution.status.success(),
        "{}",
        String::from_utf8_lossy(&execution.stderr)
    );
    assert!(String::from_utf8_lossy(&execution.stdout).contains("3\n3\n42"));
}

#[test]
fn concrete_references_implicitly_project_in_all_contextual_positions() {
    let source = format!(
        "{COUNTER}\nfunc Project(value: SimpleCounter&) -> Counter& {{ return value }}\nfunc Read(value: Counter&) -> int {{ return value.Read() }}\nfunc Main() -> int {{ var local = SimpleCounter(40); let reference = &local; let view: Counter& = reference; view.Add(2); let fromMatch: Counter& = Int32.Parse(\"1\") match {{ Ok(_) => reference, Error(_) => view }}; return Read(reference) + Read(&local) + Read(Project(reference)) + fromMatch.Read() }}"
    );
    assert_eq!(run(&source).value, Value::Int32(168));
}

#[test]
fn reference_conversions_do_not_implicitly_address_or_cross_cast() {
    assert!(program(&format!("{COUNTER}\nfunc Read(value: Counter&) -> int {{ return value.Read() }}\nfunc Main() -> int {{ var local = SimpleCounter(1); return Read(local) }}")).is_err());
    assert!(program("interface First { func Read() -> int }; interface Second { func Read() -> int }; record Cell(Value: int): First, Second { func Read() -> int { return this.Value } }; func Main() -> () { var cell = Cell(1); let first: First& = &cell; let second: Second& = first }").is_err());
}
