use neoclr::{Limits, LoadedProgram, Value, assemble, assembler::parse_function_ref, frontend};

#[test]
fn source_frame_heap_inherited_method_and_artifact_dispatch() {
    let m = frontend::compile(include_str!("../examples/source/virtual-dispatch.neo")).unwrap();
    let m = serde_json::from_str(&serde_json::to_string(&m).unwrap()).unwrap();
    let p = LoadedProgram::new(&m).unwrap();
    p.verify().unwrap();
    let result = p.run(Limits::default()).unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.heap.statistics().allocated_objects, 1);
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
        graph
            .functions
            .iter()
            .any(|f| f.target.name == "Counter.Read")
    );
}

#[test]
fn virtual_mutation_and_reference_return_use_original_derived_owner() {
    let source = "record Base(X: int) { virtual func Address() -> int& { return &this.X }; virtual func Bump() -> () { this.X = this.X + 1 } }\nrecord Child(Y: int): Base { override func Address() -> int& { return &this.Y }; override func Bump() -> () { this.Y = this.Y + 1 } }\nrecord Leaf(): Child\nfunc Main() -> int { var c = Leaf(0,40); let b: Base& = &c; b.Bump(); let y = b.Address(); y = y + 1; return c.Y }";
    let p = LoadedProgram::new(&frontend::compile(source).unwrap()).unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}

fn il(call: &str) -> LoadedProgram {
    let text = format!(
        ".module App\n.entry Main\n.type Base\n.method instance virtual byref Read() -> Int32\nldc.i4 1\nret\n.end\n.end\n.type Child\n.extends Base\n.method instance override byref Read() -> Int32\nldc.i4 42\nret\n.end\n.end\n.function Main() -> Int32\n.local Child c\nnewobj Child\nstloc c\nldloca c\ncastclass Base\n{call} instance Base::Read()\nret\n.end"
    );
    LoadedProgram::new(&assemble(&text).unwrap()).unwrap()
}

#[test]
fn call_can_select_base_body_while_callvirt_selects_override() {
    for (op, value) in [("call", 1), ("callvirt", 42)] {
        let p = il(op);
        p.verify().unwrap();
        assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(value));
    }
}

#[test]
fn overrides_require_exact_contracts_and_hiding_is_rejected() {
    for source in [
        "record A() { override func Read() -> int { return 1 } }",
        "record A() { func Read() -> int { return 1 } }\nrecord B(): A { override func Read() -> int { return 2 } }",
        "record A() { readonly virtual func Read() -> int { return 1 } }\nrecord B(): A { override func Read() -> int { return 2 } }",
        "record A() { virtual func Read() -> int { return 1 } }\nrecord B(): A { override func Read() -> bool { return true } }",
        "record A() { virtual func Read() -> int { return 1 } }\nrecord B(): A { func Read() -> int { return 2 } }",
    ] {
        assert!(
            frontend::compile(&format!("{source}\nfunc Main() -> int {{ return 0 }}")).is_err(),
            "{source}"
        );
    }
}

#[test]
fn readonly_contract_is_enforced_in_override_without_verification() {
    let text = ".module App\n.entry Main\n.type Base\n.field X Int32\n.method instance virtual readonly byref Read() -> Int32\nldarg this\nldfld 0\nret\n.end\n.end\n.type Child\n.extends Base\n.method instance override readonly byref Read() -> Int32\nldarg this\nldflda 0\nldc.i4 1\nstobj Int32\nldc.i4 0\nret\n.end\n.end\n.function Main() -> Int32\n.local Child c\nldc.i4 42\nnewobj Child\nstloc c\nldloca c\ncastclass Base\ncallvirt instance Base::Read()\nret\n.end";
    let p = LoadedProgram::new(&assemble(text).unwrap()).unwrap();
    assert!(p.verify().is_err());
    assert!(
        p.run(Limits::default())
            .unwrap_err()
            .message
            .contains("readonly")
    );
}

#[test]
fn reflection_exposes_virtual_and_override_flags_on_declared_methods() {
    let source = "record Base() { readonly virtual func Read() -> int { return 0 } }\nrecord Child(): Base { readonly override func Read() -> int { return 42 } }\nfunc Main() -> int { let baseMethods = typeof(Base).GetMethods(); let methods = typeof(Child).GetMethods(); if !baseMethods[0].IsVirtual || baseMethods[0].IsOverride { return 1 }; if methods[0].IsVirtual && methods[0].IsOverride && methods[0].IsReadOnly { return 42 }; return 0 }";
    let p = LoadedProgram::new(&frontend::compile(source).unwrap()).unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}

#[test]
fn override_reference_retains_heap_owner_under_pressure_and_frame_escape_faults() {
    let source = "record Base(X: int) { virtual func Address() -> int& { return &this.X } }\nrecord Child(Y: int): Base { override func Address() -> int& { return &this.Y } }\nfunc Make() -> int& { let b: Base& = new Child(0,42); return b.Address() }\nfunc Discard() -> () { let c = new Child(0,0) }\nfunc Main() -> int { let y = Make(); Discard(); Discard(); Discard(); return y }";
    let p = LoadedProgram::new(&frontend::compile(source).unwrap()).unwrap();
    assert_eq!(
        p.run(Limits {
            heap_objects: 2,
            ..Limits::default()
        })
        .unwrap()
        .value,
        Value::Int32(42)
    );
    let source = source.replace(
        "let b: Base& = new Child(0,42);",
        "var c = Child(0,42); let b: Base& = &c;",
    );
    let p = LoadedProgram::new(&frontend::compile(&source).unwrap()).unwrap();
    assert!(p.run(Limits::default()).is_err());
}
