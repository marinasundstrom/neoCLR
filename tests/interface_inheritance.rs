use neoclr::{Limits, LoadedProgram, Value, assemble, assembler::parse_function_ref, frontend};

#[test]
fn neo_diamond_frame_heap_readonly_and_artifact() {
    let module =
        frontend::compile(include_str!("../examples/source/interface-inheritance.neo")).unwrap();
    let restored = serde_json::from_str(&serde_json::to_string(&module).unwrap()).unwrap();
    let p = LoadedProgram::new(&restored).unwrap();
    p.verify().unwrap();
    let result = p.run(Limits::default()).unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.heap.statistics().allocated_objects, 1);
    let graph = p
        .analyze_reachability(&[parse_function_ref("Main()").unwrap()], 100)
        .unwrap();
    assert!(graph.functions.iter().any(|f| f.target.name == "Cell.Read"));
}

fn generic(body: &str) -> LoadedProgram {
    let text = format!(
        ".module App\n.entry Main\n.interface Read<T>\n.method instance Get() -> T\n.end\n.end\n.interface Child<T>\n.implements Read<T>\n.end\n.type Cell<T>\n.implements Child<T>\n.field Value T\n.method instance Get() -> T\nldarg this\nldfld 0\nret\n.end\n.end\n.function Main() -> Int32\n{body}\nret\n.end"
    );
    LoadedProgram::new(&assemble(&text).unwrap()).unwrap()
}

#[test]
fn generic_transitive_conformance_and_closed_dispatch_graph() {
    let p = generic(
        ".local Cell<Int32> x\nldc.i4 42\nnewobj Cell<Int32>\nstloc x\nldloca x\ninterface.borrow Child<Int32>\ninterface.borrow Read<Int32>\ncallvirt instance Read<Int32>::Get()",
    );
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    let graph = p
        .analyze_reachability(&[parse_function_ref("Main()").unwrap()], 100)
        .unwrap();
    assert!(graph.functions.iter().any(|f| f.target.name == "Cell.Get"));
}

#[test]
fn cycles_duplicate_bases_noninterfaces_and_missing_inherited_members_fail() {
    for declarations in [
        ".interface A\n.implements A\n.end",
        ".interface A<T>\n.implements A<A<T>>\n.end",
        ".interface A\n.implements B\n.end\n.interface B\n.implements A\n.end",
        ".interface A\n.end\n.interface B\n.implements A\n.implements A\n.end",
        ".type A\n.end\n.interface B\n.implements A\n.end",
        ".interface A\n.method instance Get() -> Int32\n.end\n.end\n.interface B\n.implements A\n.end\n.type C\n.implements B\n.end",
    ] {
        let text = format!(
            ".module App\n.entry Main\n{declarations}\n.function Main() -> Int32\nldc.i4 0\nret\n.end"
        );
        assert!(
            assemble(&text)
                .and_then(|m| LoadedProgram::new(&m))
                .is_err(),
            "{declarations}"
        );
    }
}

#[test]
fn reflection_lists_transitive_interfaces_once_excluding_self() {
    let source = "interface A {}\ninterface B: A {}\ninterface C: A {}\ninterface D: B, C {}\nrecord R(): D {}\nfunc Main() -> int { let interfaces = typeof(R).GetInterfaces(); let bases = typeof(D).GetInterfaces(); return interfaces.Length * 10 + bases.Length }";
    let p = LoadedProgram::new(&frontend::compile(source).unwrap()).unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(43));
}

#[test]
fn ambiguous_names_require_base_projection() {
    let declarations = "interface A { func Get() -> int }\ninterface B { func Get() -> int }\ninterface C: A, B {}\nrecord R(): C { func Get() -> int { return 42 } }\n";
    assert!(
        frontend::compile(&format!(
            "{declarations}func Main() -> int {{ var r = R(); let c: C& = &r; return c.Get() }}"
        ))
        .unwrap_err()
        .message
        .contains("ambiguous")
    );
    let m = frontend::compile(&format!("{declarations}func Main() -> int {{ var r = R(); let c: C& = &r; let a: A& = c; return a.Get() }}")).unwrap();
    assert_eq!(
        LoadedProgram::new(&m)
            .unwrap()
            .run(Limits::default())
            .unwrap()
            .value,
        Value::Int32(42)
    );
}

#[test]
fn readonly_cannot_upgrade_and_base_views_cannot_escape_frames() {
    let declarations = "interface A { readonly func Get() -> int }\ninterface B: A {}\nrecord R(): B { readonly func Get() -> int { return 42 } }\n";
    assert!(frontend::compile(&format!("{declarations}func Main() -> int {{ let b: readonly B& = new R(); let a: A& = b; return a.Get() }}")).is_err());
    let m = frontend::compile(&format!("{declarations}func Escape() -> A& {{ var r = R(); let b: B& = &r; return b }}\nfunc Main() -> int {{ let a = Escape(); return a.Get() }}")).unwrap();
    assert!(
        LoadedProgram::new(&m)
            .unwrap()
            .run(Limits::default())
            .is_err()
    );
}

#[test]
fn inherited_views_preserve_heap_root_and_identity_under_pressure() {
    let source = "interface A { readonly func Get() -> int }\ninterface B: A {}\nrecord R(Value: int): B { readonly func Get() -> int { return this.Value } }\nfunc Make() -> B& { return new R(42) }\nfunc Discard() -> () { let value = new R(0) }\nfunc Main() -> int { let b = Make(); let a: readonly A& = b; if !ReferenceEquals(b, a) { return 0 }; Discard(); Discard(); Discard(); return a.Get() }";
    let m = frontend::compile(source).unwrap();
    let p = LoadedProgram::new(&m).unwrap();
    p.verify().unwrap();
    assert_eq!(
        p.run(Limits {
            heap_objects: 2,
            ..Limits::default()
        })
        .unwrap()
        .value,
        Value::Int32(42)
    );
}

#[test]
fn conflicting_inherited_contracts_are_rejected_without_an_implementer() {
    for member in [
        ".method instance Get() -> Boolean",
        ".method instance readonly byref Get() -> Int32",
    ] {
        let text = format!(
            ".module App\n.entry Main\n.interface A\n.method instance Get() -> Int32\n.end\n.end\n.interface B\n{member}\n.end\n.end\n.interface C\n.implements A\n.implements B\n.end\n.function Main() -> Int32\nldc.i4 0\nret\n.end"
        );
        assert!(
            assemble(&text)
                .and_then(|m| LoadedProgram::new(&m))
                .unwrap_err()
                .message
                .contains("incompatible")
        );
    }
}

#[test]
fn unchecked_base_projection_does_not_upgrade_readonly_access() {
    let source = "interface A { readonly func Get() -> int }\ninterface B: A {}\nrecord R(): B { readonly func Get() -> int { return 42 } }\nfunc Observe(readonly b: B&) -> readonly A& { return b }\nfunc Main() -> int { let b: B& = new R(); let a = Observe(b); return a.Get() }";
    let il = frontend::lower_to_il(source)
        .unwrap()
        .replace("-> readonly A&", "-> A&");
    let p = LoadedProgram::new(&assemble(&il).unwrap()).unwrap();
    assert!(p.verify().is_err());
    assert!(
        p.run(Limits::default())
            .unwrap_err()
            .message
            .contains("readonly")
    );
}

#[test]
fn base_view_cannot_downcast_even_when_concrete_receiver_implements_child() {
    let p = generic(
        ".local Cell<Int32> x\nldc.i4 42\nnewobj Cell<Int32>\nstloc x\nldloca x\ninterface.borrow Read<Int32>\ninterface.borrow Child<Int32>\npop\nldc.i4 0",
    );
    assert!(p.verify().is_err());
    assert!(p.run(Limits::default()).is_err());
}

#[test]
fn native_base_views_keep_existing_pointer_dispatch_rules() {
    let p = generic(
        ".local Cell<Int32>* p\nsizeof Cell<Int32>\nlocalloc\nptr.cast Cell<Int32>\nstloc p\nldloc p\nldc.i4 42\nnewobj Cell<Int32>\nstobj Cell<Int32>\nldloc p\ninterface.borrow Child<Int32>\ninterface.borrow Read<Int32>\ncallvirt instance Read<Int32>::Get()",
    );
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}
