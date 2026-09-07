use neoclr::{Limits, LoadedProgram, Value, assemble};
#[test]
fn union_try_get_sample_extracts_a_typed_case() {
    let p = LoadedProgram::new(&assemble(include_str!("../examples/union_try_get.neoil")).unwrap())
        .unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().output, ["42"]);
}
#[test]
fn all_library_union_cases_support_conditional_managed_output() {
    for (carrier, case, factory) in [
        (
            "System.Option<String>",
            "System.Option.Some<String>",
            "ldstr \"hello\"\nnewobj instance System.Option.Some<String>::.ctor(String)",
        ),
        (
            "System.Option<Void>",
            "System.Option.Some<Void>",
            "ldvoid\nnewobj instance System.Option.Some<Void>::.ctor(Void)",
        ),
        (
            "System.Option<Int32>",
            "System.Option.None",
            "newobj instance System.Option.None::.ctor()",
        ),
        (
            "System.Result<Int32,String>",
            "System.Result.Ok<Int32>",
            "ldc.i4 42\nnewobj instance System.Result.Ok<Int32>::.ctor(Int32)",
        ),
        (
            "System.Result<Int32,String>",
            "System.Result.Error<String>",
            "ldstr \"error\"\nnewobj instance System.Result.Error<String>::.ctor(String)",
        ),
    ] {
        let source = format!(
            ".module App\n.entry Main\n.function Main() -> {case}\n.local {case} value\n{factory}\nnewobj instance {carrier}::.ctor({case})\nldloca value\ncall instance {carrier}::TryGet({case}&)\nbrfalse Miss\nldloc value\nret\nMiss:\nfault \"unexpected miss\"\n.end"
        );
        let p = LoadedProgram::new(&assemble(&source).unwrap()).unwrap();
        p.verify().unwrap();
        assert_eq!(
            p.run(Limits::default()).unwrap().value.ty(),
            neoclr::assembler::parse_type(case).unwrap()
        );
    }
}
#[test]
fn a_mismatched_case_does_not_initialize_or_replace_output() {
    for initial in [
        "",
        "ldc.i4 7\nnewobj instance System.Result.Ok<Int32>::.ctor(Int32)\nstloc value\n",
    ] {
        let suffix = if initial.is_empty() {
            "ldc.i4 7"
        } else {
            "ldloc value\ncall instance System.Result.Ok<Int32>::get_Value()"
        };
        let source = format!(
            ".module App\n.entry Main\n.function Main() -> Int32\n.local System.Result.Ok<Int32> value\n{initial}ldstr \"error\"\nnewobj instance System.Result.Error<String>::.ctor(String)\nnewobj instance System.Result<Int32,String>::.ctor(System.Result.Error<String>)\nldloca value\ncall instance System.Result<Int32,String>::TryGet(System.Result.Ok<Int32>&)\nbrfalse Miss\nfault \"unexpected match\"\nMiss:\n{suffix}\nret\n.end"
        );
        let p = LoadedProgram::new(&assemble(&source).unwrap()).unwrap();
        p.verify().unwrap();
        assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(7));
    }
}

#[test]
fn case_reference_overloads_remain_distinct_when_payload_types_are_identical() {
    let source = ".module App\n.entry Main\n.function Main() -> Int32\n.local System.Result<Int32,Int32> result\n.local System.Result.Ok<Int32> ok\n.local System.Result.Error<Int32> error\nldc.i4 42\nnewobj instance System.Result.Ok<Int32>::.ctor(Int32)\nnewobj instance System.Result<Int32,Int32>::.ctor(System.Result.Ok<Int32>)\nstloc result\nldloc result\nldloca error\ncall instance System.Result<Int32,Int32>::TryGet(System.Result.Error<Int32>&)\nbrfalse GetOk\nfault \"wrong case matched\"\nGetOk:\nldloc result\nldloca ok\ncall instance System.Result<Int32,Int32>::TryGet(System.Result.Ok<Int32>&)\nbrfalse Miss\nldloc ok\ncall instance System.Result.Ok<Int32>::get_Value()\nret\nMiss:\nfault \"expected Ok\"\n.end";
    let module = assemble(source).unwrap();
    let module = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    let p = LoadedProgram::new(&module).unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    let graph = p
        .analyze_reachability(
            &[neoclr::assembler::parse_function_ref("Main()").unwrap()],
            40,
        )
        .unwrap();
    let methods: Vec<_> = graph
        .functions
        .iter()
        .filter(|f| f.target.name == "System.Result.TryGet")
        .collect();
    assert_eq!(methods.len(), 2);
    assert_ne!(methods[0].target.definition, methods[1].target.definition);
    assert!(methods.iter().all(|f| f.out_when_true == [0]));
}
