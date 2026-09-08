use neoclr::{Limits, LoadedProgram, Value, assembler::parse_function_ref, frontend};

fn program(source: &str) -> LoadedProgram {
    LoadedProgram::new(&frontend::compile(source).unwrap()).unwrap()
}

#[test]
fn frame_heap_abstract_base_and_closed_graph_roundtrip() {
    let module =
        frontend::compile(include_str!("../examples/source/class-interfaces.neo")).unwrap();
    let module = serde_json::from_str(&serde_json::to_string(&module).unwrap()).unwrap();
    let p = LoadedProgram::new(&module).unwrap();
    p.verify().unwrap();
    let result = p.run(Limits::default()).unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.heap.statistics().allocated_objects, 1);
    let graph = p
        .analyze_reachability(&[parse_function_ref("Main()").unwrap()], 100)
        .unwrap();
    for name in ["OffsetCounter.Read", "CounterBase.Add"] {
        assert!(graph.functions.iter().any(|f| f.target.name == name));
    }
    assert!(
        !graph
            .functions
            .iter()
            .any(|f| f.target.name == "CounterBase.Read")
    );
}

#[test]
fn new_and_redeclared_conformance_can_use_inherited_public_body() {
    for base_contract in ["", ": Readable"] {
        let p = program(&format!(
            "interface Readable {{ func Read() -> int }}\nrecord Base(X: int){base_contract} {{ func Read() -> int {{ return this.X }} }}\nrecord Child(): Base, Readable\nfunc Main() -> int {{ var c = Child(42); let r: Readable& = &c; return r.Read() }}"
        ));
        p.verify().unwrap();
        assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    }
}

#[test]
fn override_reference_from_base_interface_retains_complete_heap_owner() {
    let declarations = "interface Addressable { func Address() -> int& }\nrecord Base(X: int): Addressable { virtual func Address() -> int& { return &this.X } }\nrecord Child(Y: int): Base { override func Address() -> int& { return &this.Y } }\nrecord Leaf(): Child\n";
    let p = program(&format!(
        "{declarations}func Make() -> int& {{ let b: Base& = new Leaf(0,41); let a: Addressable& = b; return a.Address() }}\nfunc Discard() -> () {{ let c = new Leaf(0,0) }}\nfunc Main() -> int {{ let y = Make(); Discard(); Discard(); Discard(); y = y + 1; return y }}"
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
        "{declarations}func Make() -> int& {{ var c = Leaf(0,42); let b: Base& = &c; let a: Addressable& = b; return a.Address() }}\nfunc Main() -> int {{ return Make() }}"
    ));
    assert!(p.run(Limits::default()).is_err());
}

#[test]
fn inherited_interface_diamonds_are_deduplicated_in_reflection() {
    let p = program(
        "interface Root {}\ninterface Left: Root {}\ninterface Right: Root {}\nrecord Base(): Left\nrecord Child(): Base, Right\nfunc Main() -> int { return typeof(Child).GetInterfaces().Length }",
    );
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(3));
}

#[test]
fn incomplete_or_incompatible_contracts_and_hidden_capabilities_are_rejected() {
    for source in [
        "interface Readable { func Read() -> int }\nabstract record Base(): Readable {}",
        "interface Readable { readonly func Read() -> int }\nrecord Base() { func Read() -> int { return 42 } }\nrecord Child(): Base, Readable",
        "interface Readable { func Read() -> int }\nabstract record Base(): Readable { abstract func Read() -> int }\nrecord Child(): Base",
    ] {
        assert!(
            frontend::compile(&format!("{source}\nfunc Main() -> int {{ return 0 }}")).is_err(),
            "{source}"
        );
    }
    assert!(frontend::compile("interface Extra {}\nrecord Base()\nrecord Child(): Base, Extra\nfunc Main() -> int { let b: Base& = new Child(); let e: Extra& = b; return 0 }").is_err());
}

#[test]
fn generic_base_mapping_infers_closed_override_target() {
    let text = ".module App\n.entry Main\n.interface Read<T>\n.method instance byref Get() -> T\n.end\n.end\n.type abstract Base<T>\n.implements Read<T>\n.field X T\n.method instance abstract byref Get() -> T\n.end\n.end\n.type Child<T>\n.extends Base<T>\n.method instance override byref Get() -> T\nldarg this\nldfld 0\nret\n.end\n.end\n.function Main() -> Int32\n.local Child<Int32> c\nldc.i4 42\nnewobj Child<Int32>\nstloc c\nldloca c\ncastclass Base<Int32>\ninterface.borrow Read<Int32>\ncallvirt instance Read<Int32>::Get()\nret\n.end";
    let p = LoadedProgram::new(&neoclr::assemble(text).unwrap()).unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    let graph = p
        .analyze_reachability(&[parse_function_ref("Main()").unwrap()], 100)
        .unwrap();
    assert!(graph.functions.iter().any(|f| f.target.name == "Child.Get"));
    assert!(!graph.functions.iter().any(|f| f.target.name == "Base.Get"));
}

#[test]
fn readonly_interface_override_cannot_write_without_verification() {
    let text = ".module App\n.entry Main\n.interface Readable\n.method instance readonly byref Read() -> Int32\n.end\n.end\n.type Base\n.implements Readable\n.field X Int32\n.method instance virtual readonly byref Read() -> Int32\nldarg this\nldfld 0\nret\n.end\n.end\n.type Child\n.extends Base\n.method instance override readonly byref Read() -> Int32\nldarg this\nldflda 0\nldc.i4 1\nstobj Int32\nldc.i4 0\nret\n.end\n.end\n.function Main() -> Int32\n.local Child c\nldc.i4 42\nnewobj Child\nstloc c\nldloca c\ncastclass Base\ninterface.borrow Readable\ncallvirt instance Readable::Read()\nret\n.end";
    let p = LoadedProgram::new(&neoclr::assemble(text).unwrap()).unwrap();
    assert!(p.verify().is_err());
    assert!(
        p.run(Limits::default())
            .unwrap_err()
            .message
            .contains("readonly")
    );
}
