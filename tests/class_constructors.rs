use neoclr::{Limits, Value, assemble, run, verify};
const TYPE: &str = ".type class Counter\n.field Age Int32\n.method instance .ctor(Int32 age) -> noresult\nldarg 0\nldarg age\nstfld Counter::Age\nret\n.end\n.method instance Set(Int32 age) -> noresult\nldarg 0\nldarg age\nstfld Counter::Age\nret\n.end\n.method instance Get() -> Int32\nldarg 0\nldfld Counter::Age\nret\n.end\n.end\n";
fn module(main: &str) -> neoclr::Module {
    assemble(&format!(
        ".module Constructors\n.entry Main\n{TYPE}.function Main() -> Int32\n{main}\n.end"
    ))
    .unwrap()
}
#[test]
fn constructor_result_and_empty_instance_calls_follow_clr_stack_behavior() {
    let m = module(
        ".local Counter c\nldc.i4 1\nnewobj instance Counter::.ctor(Int32)\nstloc c\nldc.i4 100\nldloc c\nldc.i4 42\ncall instance Counter::Set(Int32)\nldloc c\ncall instance Counter::Get()\nadd\nret",
    );
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(142));
}
#[test]
fn default_primitive_fields_are_readable_in_constructor() {
    let m = module(
        "ldc.i4 8\nnewobj instance Counter::.ctor(Int32)\ncall instance Counter::Get()\nret",
    );
    let mut m = m;
    let ctor = m
        .functions
        .iter_mut()
        .find(|f| f.name.ends_with("..ctor"))
        .unwrap();
    // Read Age before assignment; ordinary class allocation initializes supported fields.
    ctor.body = vec![
        neoclr::metadata::Instruction::Arg(0),
        neoclr::metadata::Instruction::Field(0),
        neoclr::metadata::Instruction::Pop,
        neoclr::metadata::Instruction::Return,
    ];
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(0));
}
#[test]
fn extra_stack_values_in_ctor_or_after_class_field_store_are_rejected() {
    let m = module(
        "ldc.i4 1\nnewobj instance Counter::.ctor(Int32)\ncall instance Counter::Get()\nret",
    );
    for op in [
        neoclr::metadata::Instruction::Void,
        neoclr::metadata::Instruction::Pop,
    ] {
        let mut m = m.clone();
        let ctor = m
            .functions
            .iter_mut()
            .find(|f| f.name.ends_with("..ctor"))
            .unwrap();
        ctor.body.insert(3, op);
        assert!(verify(&m).is_err());
        assert!(run(&m, Limits::default()).is_err());
    }
}
#[test]
fn failed_constructor_does_not_return_an_object() {
    let mut m = module(
        "ldc.i4 1\nnewobj instance Counter::.ctor(Int32)\ncall instance Counter::Get()\nret",
    );
    m.functions
        .iter_mut()
        .find(|f| f.name.ends_with("..ctor"))
        .unwrap()
        .body = vec![neoclr::metadata::Instruction::Fault(
        "constructor failed".into(),
    )];
    verify(&m).unwrap();
    assert!(
        run(&m, Limits::default())
            .unwrap_err()
            .message
            .contains("constructor failed")
    );
}
#[test]
fn constructor_allocation_obeys_budget() {
    let m = module(
        "ldc.i4 1\nnewobj instance Counter::.ctor(Int32)\ncall instance Counter::Get()\nret",
    );
    assert!(
        run(
            &m,
            Limits {
                heap_objects: 0,
                ..Limits::default()
            }
        )
        .unwrap_err()
        .message
        .contains("heap object limit")
    );
}

#[test]
fn newobj_retains_original_object_if_this_slot_changes_during_collection() {
    let mut m = module(
        "ldc.i4 1\nnewobj instance Counter::.ctor(Int32)\ncall instance Counter::Get()\nret",
    );
    use neoclr::metadata::{Instruction as Op, Type};
    let ty = Type::Named("Counter".into());
    m.functions
        .iter_mut()
        .find(|f| f.name.ends_with("..ctor"))
        .unwrap()
        .body = vec![
        Op::Int(9),
        Op::New(ty.clone()),
        Op::StoreArg(0),
        Op::Int(8),
        Op::New(ty.clone()),
        Op::Pop,
        Op::Int(7),
        Op::New(ty),
        Op::Pop,
        Op::Return,
    ];
    verify(&m).unwrap();
    let execution = run(
        &m,
        Limits {
            heap_objects: 3,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(execution.value, Value::Int32(0));
    assert!(execution.heap.collections() >= 2);
}

#[test]
fn direct_constructor_calls_are_rejected_until_chaining_is_supported() {
    let mut m = module(
        "ldc.i4 1\nnewobj instance Counter::.ctor(Int32)\ncall instance Counter::Get()\nret",
    );
    let ctor = m
        .functions
        .iter_mut()
        .find(|f| f.name.ends_with("..ctor"))
        .unwrap();
    ctor.no_result = false;
    assert!(verify(&m).unwrap_err().message.contains("no-result"));
    assert!(run(&m, Limits::default()).is_err());
    let source = format!(
        ".module Direct\n.entry Main\n{TYPE}.function Main() -> noresult\nldc.i4 1\nnewobj Counter\nldc.i4 2\ncall instance Counter::.ctor(Int32)\nret\n.end"
    );
    assert!(assemble(&source).unwrap_err().message.contains("chaining"));
}

#[test]
fn documented_example_runs_against_system_console() {
    let m = assemble(include_str!("../examples/class_construction.neoil")).unwrap();
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().output, ["42"]);
}
