use neoclr::{Limits, LoadedProgram, Value, assemble, assembler::parse_function_ref, frontend};

#[test]
fn abstract_base_runs_concrete_frame_heap_overrides_and_roundtrips() {
    let m = frontend::compile(include_str!("../examples/source/abstract-classes.neo")).unwrap();
    let m = serde_json::from_str(&serde_json::to_string(&m).unwrap()).unwrap();
    let p = LoadedProgram::new(&m).unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    let graph = p
        .analyze_reachability(&[parse_function_ref("Main()").unwrap()], 100)
        .unwrap();
    assert!(
        graph
            .functions
            .iter()
            .any(|f| f.target.name == "OffsetCounter.Read")
    );
    assert!(
        !graph
            .functions
            .iter()
            .any(|f| f.target.name == "Counter.Read")
    );
}

#[test]
fn abstract_classes_cannot_be_constructed_defaulted_or_imported_as_values() {
    for source in [
        "abstract record A()\nfunc Main() -> int { let a = A(); return 0 }",
        "abstract record A()\nfunc Main() -> int { let a = new A(); return 0 }",
        "abstract record A()\nfunc Main() -> int { let a = new A[1]; return 0 }",
    ] {
        assert!(frontend::compile(source).is_err(), "{source}");
    }
    for body in [
        "newobj A\npop",
        ".local A a\nldloca a\ninitobj A",
        "sizeof A\npop",
    ] {
        let text = format!(
            ".module App\n.entry Main\n.type abstract A\n.end\n.function Main() -> Int32\n{body}\nldc.i4 0\nret\n.end"
        );
        assert!(
            assemble(&text)
                .and_then(|m| LoadedProgram::new(&m))
                .and_then(|p| p.run(Limits::default()))
                .is_err()
        );
    }
    let m = assemble(".module App\n.entry Main\n.type abstract A\n.end\n.function Accept(A a) -> Int32\nldc.i4 0\nret\n.end\n.function Main() -> Int32\nldc.i4 0\nret\n.end").unwrap();
    assert!(
        LoadedProgram::new(&m)
            .unwrap()
            .resolve_function(&parse_function_ref("Accept(A)").unwrap())
            .is_err()
    );
}

#[test]
fn missing_implementations_and_abstract_methods_on_concrete_types_fail() {
    for source in [
        "record A() { abstract func Read() -> int }",
        "abstract record A() { abstract func Read() -> int }\nrecord B(): A",
        "abstract record A() { abstract func Read() -> int }\nabstract record B(): A\nrecord C(): B",
        "abstract record A() { abstract func Read() -> int }\nrecord B(): A { func Read() -> int { return 1 } }",
    ] {
        assert!(
            frontend::compile(&format!("{source}\nfunc Main() -> int {{ return 0 }}")).is_err(),
            "{source}"
        );
    }
}

#[test]
fn abstract_intermediate_can_defer_or_reabstract_implementation() {
    let source = "record A() { readonly virtual func Read() -> int { return 1 } }\nabstract record B(): A { readonly abstract override func Read() -> int }\nrecord C(): B { readonly override func Read() -> int { return 42 } }\nfunc Main() -> int { let a: A& = new C(); return a.Read() }";
    let p = LoadedProgram::new(&frontend::compile(source).unwrap()).unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}

#[test]
fn abstract_reflection_and_direct_calls_have_explicit_contracts() {
    let source = "abstract record A() { readonly abstract func Read() -> int }\nrecord B(): A { readonly override func Read() -> int { return 42 } }\nfunc Main() -> int { let methods = typeof(A).GetMethods(); if typeof(A).IsAbstract && !typeof(B).IsAbstract && methods[0].IsAbstract && methods[0].IsVirtual { let b: A& = new B(); return b.Read() }; return 0 }";
    let p = LoadedProgram::new(&frontend::compile(source).unwrap()).unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    let text = frontend::lower_to_il(source)
        .unwrap()
        .replace("callvirt instance A::Read", "call instance A::Read");
    assert!(assemble(&text).is_err());
}

#[test]
fn generic_abstract_contract_dispatches_but_open_call_graph_is_conservative() {
    let text = ".module App\n.entry Main\n.type abstract Base<T>\n.method instance abstract byref Read() -> T\n.end\n.end\n.type Child<T>\n.extends Base<T>\n.field Value T\n.method instance override byref Read() -> T\nldarg this\nldfld 0\nret\n.end\n.end\n.function Main() -> Int32\n.local Child<Int32> c\nldc.i4 42\nnewobj Child<Int32>\nstloc c\nldloca c\ncastclass Base<Int32>\ncallvirt instance Base<Int32>::Read()\nret\n.end";
    let p = LoadedProgram::new(&assemble(text).unwrap()).unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    assert!(
        p.analyze_reachability(&[parse_function_ref("Main()").unwrap()], 100)
            .unwrap_err()
            .message
            .contains("generic")
    );
}
