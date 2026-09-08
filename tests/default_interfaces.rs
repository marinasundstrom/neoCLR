use neoclr::{Limits, LoadedProgram, Value, assemble, assembler::parse_function_ref, frontend};
fn program(source: &str) -> LoadedProgram {
    LoadedProgram::new(&frontend::compile(source).unwrap()).unwrap()
}
#[test]
fn source_artifact_readonly_frame_heap_nested_dispatch_and_graph() {
    let m = frontend::compile(include_str!("../examples/source/default-interfaces.neo")).unwrap();
    let m = serde_json::from_str(&serde_json::to_string(&m).unwrap()).unwrap();
    let p = LoadedProgram::new(&m).unwrap();
    p.verify().unwrap();
    let result = p.run(Limits::default()).unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.heap.statistics().allocated_objects, 1);
    let graph = p
        .analyze_reachability(&[parse_function_ref("Main()").unwrap()], 100)
        .unwrap();
    for name in ["Readable.Twice", "Counter.Readable.Read"] {
        assert!(graph.functions.iter().any(|f| f.target.name == name));
    }
}
const DIAMOND: &str = "interface Root { readonly func Read() -> int { return 1 } }\ninterface Left: Root { readonly func Root.Read() -> int { return 2 } }\ninterface Right: Root { readonly func Root.Read() -> int { return 3 } }\ninterface Both: Left, Right { readonly func Root.Read() -> int { return 42 } }\n";
#[test]
fn most_specific_diamonds_and_class_precedence() {
    for (decl, expected) in [
        ("record C(): Root", 1),
        ("record C(): Left", 2),
        ("record C(): Both", 42),
        (
            "record C(): Left, Right { readonly func Read() -> int { return 42 } }",
            42,
        ),
        (
            "record C(): Left, Right { readonly func Root.Read() -> int { return 42 } }",
            42,
        ),
    ] {
        let p = program(&format!(
            "{DIAMOND}{decl}\nfunc Main() -> int {{ let c = new C(); let r: readonly Root& = c; return r.Read() }}"
        ));
        p.verify().unwrap();
        assert_eq!(
            p.run(Limits::default()).unwrap().value,
            Value::Int32(expected)
        );
    }
    for bases in ["Left, Right", "Right, Left"] {
        assert!(
            frontend::compile(&format!(
                "{DIAMOND}record C(): {bases}\nfunc Main() -> int {{ return 0 }}"
            ))
            .is_err()
        );
    }
}
#[test]
fn reabstraction_requires_class_implementation_and_invalid_contracts_do_not_fallback() {
    let prefix = "interface Root { readonly func Read() -> int { return 1 } }\ninterface Required: Root { readonly abstract func Root.Read() -> int }\n";
    assert!(
        frontend::compile(&format!(
            "{prefix}record C(): Required\nfunc Main() -> int {{ return 0 }}"
        ))
        .is_err()
    );
    let p = program(&format!(
        "{prefix}record C(): Required {{ readonly func Read() -> int {{ return 42 }} }}\nfunc Main() -> int {{ let r: readonly Root& = new C(); return r.Read() }}"
    ));
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    for decl in [
        "record C(): Root { func Read() -> int { return 42 } }",
        "abstract record Base(): Root { readonly abstract func Read() -> int }; record C(): Base",
    ] {
        assert!(
            frontend::compile(&format!(
                "{prefix}{decl}\nfunc Main() -> int {{ return 0 }}"
            ))
            .is_err()
        );
    }
}
#[test]
fn inherited_mapping_stays_anchored_and_redeclaration_restarts_search() {
    for (extra, expected) in [("", 1), (", Root", 42)] {
        let p = program(&format!(
            "interface Root {{ func Read() -> int {{ return 1 }} }}\nrecord Base(): Root\nrecord Child(): Base{extra} {{ func Read() -> int {{ return 42 }} }}\nfunc Main() -> int {{ let b: Base& = new Child(); let r: Root& = b; return r.Read() }}"
        ));
        p.verify().unwrap();
        assert_eq!(
            p.run(Limits::default()).unwrap().value,
            Value::Int32(expected)
        );
    }
}
#[test]
fn default_reference_return_retains_heap_owner_and_frame_escape_faults() {
    let prefix = "interface Addressable { func Address() -> int&; func Forward() -> int& { return this.Address() } }\nrecord C(X: int): Addressable { func Addressable.Address() -> int& { return &this.X } }\n";
    let p = program(&format!(
        "{prefix}func Make() -> int& {{ let a: Addressable& = new C(41); return a.Forward() }}\nfunc Discard() -> () {{ let c = new C(0) }}\nfunc Main() -> int {{ let x = Make(); Discard(); Discard(); x = x + 1; return x }}"
    ));
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
    let p = program(&format!(
        "{prefix}func Make() -> int& {{ var c = C(42); let a: Addressable& = &c; return a.Forward() }}\nfunc Main() -> int {{ return Make() }}"
    ));
    assert!(p.run(Limits::default()).is_err());
}
#[test]
fn reflection_distinguishes_default_body_and_abstract_contract_without_class_rows() {
    let p = program(
        "interface R { readonly func Read() -> int; readonly func Twice() -> int { return this.Read() + this.Read() } }\nrecord C(): R { readonly func Read() -> int { return 21 } }\nfunc Main() -> int { let methods = typeof(R).GetMethods(); if !methods[0].IsAbstract || methods[1].IsAbstract || !methods[1].IsVirtual { return 0 }; return typeof(C).GetMethods().Length + 41 }",
    );
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    assert!(frontend::compile("interface R { func Read() -> int { return 42 } }\nrecord C(): R\nfunc Main() -> int { var c = C(); return c.Read() }").is_err());
}
#[test]
fn generic_default_body_and_closed_dynamic_targets() {
    let m=assemble(".module App\n.entry Main\n.interface Read<T>\n.method instance byref Get() -> T\n.end\n.method instance byref Forward() -> T\nldarg this\ncallvirt instance Read<T>::Get()\nret\n.end\n.end\n.type C<T>\n.implements Read<T>\n.field X T\n.method instance byref Get() -> T\nldarg this\nldfld 0\nret\n.end\n.end\n.function Main() -> Int32\n.local C<Int32> c\nldc.i4 42\nnewobj C<Int32>\nstloc c\nldloca c\ninterface.borrow Read<Int32>\ncallvirt instance Read<Int32>::Forward()\nret\n.end").unwrap();
    let p = LoadedProgram::new(&m).unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    let g = p
        .analyze_reachability(&[parse_function_ref("Main()").unwrap()], 100)
        .unwrap();
    assert!(g.functions.iter().any(|f| f.target.name == "Read.Forward"));
    assert!(g.functions.iter().any(|f| f.target.name == "C.Get"));
}

#[test]
fn readonly_default_cannot_call_writable_contract_without_verification() {
    let m=assemble(".module App\n.entry Main\n.interface R\n.method instance byref Write() -> Int32\n.end\n.method instance readonly byref Read() -> Int32\nldarg this\ncallvirt instance R::Write()\nret\n.end\n.end\n.type C\n.implements R\n.method instance byref Write() -> Int32\nldc.i4 42\nret\n.end\n.end\n.function Main() -> Int32\n.local C c\nnewobj C\nstloc c\nldloca c\ninterface.borrow R\ncallvirt instance R::Read()\nret\n.end").unwrap();
    let p = LoadedProgram::new(&m).unwrap();
    assert!(p.verify().is_err());
    assert!(
        p.run(Limits::default())
            .unwrap_err()
            .message
            .contains("readonly")
    );
}
#[test]
fn output_default_and_identity_return_preserve_original_receiver() {
    let p = program(
        "interface Writer { func Write(out x: int&) -> () { x = 42 }; func Same() -> Writer& { return this } }\nrecord C(): Writer\nfunc Main() -> int { var c = C(); let w: Writer& = &c; let same = w.Same(); if !ReferenceEquals(w, same) { return 0 }; var x: int; same.Write(out x); return x }",
    );
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    let mut m=frontend::compile("interface Writer { func Write(out x: int&) -> () { x = 42 } }\nrecord C(): Writer\nfunc Main() -> int { var c = C(); let w: Writer& = &c; var x: int; w.Write(out x); return x }").unwrap();
    let method = m
        .functions
        .iter_mut()
        .find(|f| f.name == "Writer.Write")
        .unwrap();
    method.sequence_points.clear();
    method.body = vec![
        neoclr::metadata::Instruction::Void,
        neoclr::metadata::Instruction::Return,
    ];
    let p = LoadedProgram::new(&m).unwrap();
    // As with class bodies, output completion is checked on runtime return.
    assert!(p.run(Limits::default()).is_err());
}
#[test]
fn selected_default_fault_trace_has_executed_body_and_source() {
    let source = "interface Root { func Read() -> int { return 1 } }\ninterface Child: Root { func Root.Read() -> int { return 1 / 0 } }\nrecord C(): Child\nfunc Main() -> int { let r: Root& = new C(); return r.Read() }";
    let m = frontend::compile_named(source, "defaults.neo").unwrap();
    let p = LoadedProgram::new(&m).unwrap();
    p.verify().unwrap();
    let fault = p.run(Limits::default()).unwrap_err();
    let frames = fault.stack_trace.unwrap().frames;
    assert_eq!(frames[0].function.name, "Child.Root.Read");
    assert_eq!(frames[0].source.as_ref().unwrap().line, 2);
}

#[test]
fn complete_owner_selects_derived_interface_body_through_class_base_view() {
    let p = program(
        "interface Root { func Read() -> int { return 1 } }\ninterface Child: Root { func Root.Read() -> int { return 42 } }\nrecord Base(): Root\nrecord C(): Base, Child\nfunc Main() -> int { let b: Base& = new C(); let r: Root& = b; return r.Read() }",
    );
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}
#[test]
fn default_bodies_require_managed_receivers_and_virtual_contract_calls() {
    for method in [
        ".method instance Read() -> Int32\nldc.i4 42\nret\n.end",
        ".method instance abstract byref Read() -> Int32\nldc.i4 42\nret\n.end",
    ] {
        assert!(assemble(&format!(".module App\n.interface R\n{method}\n.end")).is_err());
    }
    let source = "interface R { func Read() -> int { return 42 } }\nrecord C(): R\nfunc Main() -> int { let r: R& = new C(); return r.Read() }";
    let mut m = frontend::compile(source).unwrap();
    let main = m.functions.iter_mut().find(|f| f.name == "Main").unwrap();
    for op in &mut main.body {
        if let neoclr::metadata::Instruction::CallVirtual(target) = op {
            *op = neoclr::metadata::Instruction::Call(target.clone());
        }
    }
    assert!(LoadedProgram::new(&m).is_err());
}
