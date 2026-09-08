use neoclr::{Limits, LoadedProgram, Value, assemble, assembler::parse_function_ref, frontend};

fn source(text: &str) -> LoadedProgram {
    LoadedProgram::new(&frontend::compile(text).unwrap()).unwrap()
}

#[test]
fn source_roundtrip_inherited_frame_heap_views_and_graph() {
    let m = frontend::compile(include_str!("../examples/source/explicit-interfaces.neo")).unwrap();
    let m = serde_json::from_str(&serde_json::to_string(&m).unwrap()).unwrap();
    let p = LoadedProgram::new(&m).unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    let graph = p
        .analyze_reachability(&[parse_function_ref("Main()").unwrap()], 100)
        .unwrap();
    for name in ["Counter.Readable.Read", "Counter.Identified.Read"] {
        assert!(graph.functions.iter().any(|f| f.target.name == name));
    }
}

#[test]
fn explicit_mapping_is_separate_from_public_virtual_slot_and_can_be_reimplemented() {
    for (declaration, expected) in [("", 21), (", Readable", 42)] {
        let p = source(&format!(
            "interface Readable {{ func Read() -> int }}\nrecord Base(): Readable {{ func Readable.Read() -> int {{ return 21 }}; virtual func Read() -> int {{ return 0 }} }}\nrecord Child(): Base{declaration} {{ override func Read() -> int {{ return 42 }} }}\nfunc Main() -> int {{ var c = Child(); let b: Base& = &c; let r: Readable& = b; if c.Read() != 42 {{ return 0 }}; return r.Read() }}"
        ));
        p.verify().unwrap();
        assert_eq!(
            p.run(Limits::default()).unwrap().value,
            Value::Int32(expected)
        );
    }
    let p = source(
        "interface Readable { func Read() -> int }\nrecord Base(): Readable { func Readable.Read() -> int { return 1 } }\nrecord Child(): Base, Readable { func Readable.Read() -> int { return 42 } }\nfunc Main() -> int { let b: Base& = new Child(); let r: Readable& = b; return r.Read() }",
    );
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}

fn il(body: &str, main: &str) -> neoclr::Module {
    il_result(body, main).unwrap()
}
fn il_result(body: &str, main: &str) -> Result<neoclr::Module, neoclr::Fault> {
    assemble(&format!(
        ".module App\n.entry Main\n.interface Readable\n.method instance byref Read() -> Int32\n.end\n.end\n.type Cell\n.implements Readable\n{body}\n.end\n.function Main() -> Int32\n.local Cell c\nnewobj Cell\nstloc c\nldloca c\n{main}\nret\n.end"
    ))
}
const BODY: &str = ".method private instance byref Hidden() -> Int32\n.override instance Readable::Read()\nldc.i4 42\nret\n.end";
const CALL: &str = "interface.borrow Readable\ncallvirt instance Readable::Read()";

#[test]
fn mapping_uses_declaration_identity_not_body_name_and_direct_calls_are_private() {
    let m = il(BODY, CALL);
    let p = LoadedProgram::new(&m).unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    assert!(il_result(BODY, "call instance Cell::Hidden()").is_err());
    assert!(frontend::compile("interface Readable { func Read() -> int }\nrecord C(): Readable { func Readable.Read() -> int { return 42 } }\nfunc Main() -> int { var c = C(); return c.Read() }").is_err());
}

#[test]
fn malformed_mapping_artifacts_are_rejected_on_load() {
    for variant in 0..8 {
        let mut m = il(BODY, CALL);
        let body = m
            .functions
            .iter_mut()
            .find(|f| f.name == "Cell.Hidden")
            .unwrap();
        match variant {
            0 => body
                .interface_implementations
                .push(body.interface_implementations[0].clone()),
            1 => body.visibility = neoclr::metadata::Visibility::Public,
            2 => body.receiver_readonly = true,
            3 => body.is_virtual = true,
            4 => body.interface_implementations[0].name = "Readable.Missing".into(),
            5 => {
                body.interface_implementations[0] =
                    parse_function_ref("instance Cell::Hidden()").unwrap()
            }
            6 => body.interface_implementations[0]
                .parameters
                .push(neoclr::metadata::Type::Int32),
            7 => m
                .types
                .iter_mut()
                .find(|t| t.name == "Cell")
                .unwrap()
                .implements
                .clear(),
            _ => unreachable!(),
        }
        assert!(LoadedProgram::new(&m).is_err(), "variant {variant}");
    }
    let duplicate = format!("{BODY}\n{}", BODY.replace("Hidden", "Other"));
    assert!(il_result(&duplicate, CALL).is_err());
}

#[test]
fn generic_explicit_mapping_substitution_and_closed_analysis() {
    let m = assemble(".module App\n.entry Main\n.interface Read<T>\n.method instance byref Get() -> T\n.end\n.end\n.type Base<T>\n.implements Read<T>\n.field X T\n.method private instance byref Hidden() -> T\n.override instance Read<T>::Get()\nldarg this\nldfld 0\nret\n.end\n.end\n.type Child<T>\n.extends Base<T>\n.end\n.function Main() -> Int32\n.local Child<Int32> c\nldc.i4 42\nnewobj Child<Int32>\nstloc c\nldloca c\ninterface.borrow Read<Int32>\ncallvirt instance Read<Int32>::Get()\nret\n.end").unwrap();
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
            .any(|f| f.target.name == "Base.Hidden")
    );
}

#[test]
fn explicit_body_reference_return_retains_heap_owner_and_rejects_frame_escape() {
    let declarations = "interface Addressable { func Address() -> int& }\nrecord Base(X: int): Addressable { func Addressable.Address() -> int& { return &this.X } }\nrecord Child(Y: int): Base\n";
    let p = source(&format!(
        "{declarations}func Make() -> int& {{ let b: Base& = new Child(41,0); let a: Addressable& = b; return a.Address() }}\nfunc Discard() -> () {{ let c = new Child(0,0) }}\nfunc Main() -> int {{ let x = Make(); Discard(); Discard(); x = x + 1; return x }}"
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
    let p = source(&format!(
        "{declarations}func Make() -> int& {{ var c = Child(42,0); let a: Addressable& = &c; return a.Address() }}\nfunc Main() -> int {{ return Make() }}"
    ));
    assert!(p.run(Limits::default()).is_err());
}

#[test]
fn reflection_keeps_qualified_private_names_and_public_surface_clean() {
    let p = source(
        "interface Readable { func Read() -> int }\nrecord C(): Readable { func Readable.Read() -> int { return 42 } }\nfunc Main() -> int { let publicMethods = typeof(C).GetMethods(); let hidden = typeof(C).GetMethods(System.Reflection.BindingFlags.FromValue(36)); if publicMethods.Length != 0 { return 0 }; if hidden.Length == 1 && hidden[0].IsPrivate && hidden[0].Name.Equals(\"Readable.Read\") { return 42 }; return 0 }",
    );
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}

#[test]
fn explicit_readonly_and_output_contracts_survive_dispatch() {
    let p = source(
        "interface Writer { func Set(out value: int&) -> () }\nrecord C(): Writer { func Writer.Set(out value: int&) -> () { value = 42 } }\nfunc Main() -> int { var c = C(); let w: Writer& = &c; var result: int; w.Set(out result); return result }",
    );
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    let mut m = il(BODY, CALL);
    for f in &mut m.functions {
        if f.name == "Readable.Read" || f.name == "Cell.Hidden" {
            f.receiver_readonly = true;
        }
    }
    // A whole-value write attempts to bypass the source readonly restrictions.
    let body = m
        .functions
        .iter_mut()
        .find(|f| f.name == "Cell.Hidden")
        .unwrap();
    body.body = vec![
        neoclr::metadata::Instruction::Arg(0),
        neoclr::metadata::Instruction::New(neoclr::metadata::Type::from_name("Cell")),
        neoclr::metadata::Instruction::StoreObject(neoclr::metadata::Type::from_name("Cell")),
        neoclr::metadata::Instruction::Int(42),
        neoclr::metadata::Instruction::Return,
    ];
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
fn redeclaration_searches_each_ancestor_for_explicit_then_public_members() {
    let p = source(
        "interface Readable { func Read() -> int }\nrecord Base(): Readable { func Readable.Read() -> int { return 21 }; virtual func Read() -> int { return 0 } }\nrecord Middle(): Base { override func Read() -> int { return 42 } }\nrecord Leaf(): Middle, Readable\nfunc Main() -> int { let b: Base& = new Leaf(); let r: Readable& = b; return r.Read() }",
    );
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}

#[test]
fn generic_mapping_collisions_after_substitution_fault() {
    let m = assemble(".module App\n.entry Main\n.interface Read<T>\n.method instance byref Get() -> Int32\n.end\n.end\n.type C<T>\n.implements Read<T>\n.implements Read<Int32>\n.method private instance byref A() -> Int32\n.override instance Read<T>::Get()\nldc.i4 1\nret\n.end\n.method private instance byref B() -> Int32\n.override instance Read<Int32>::Get()\nldc.i4 2\nret\n.end\n.end\n.function Main() -> Int32\n.local C<Int32> c\nnewobj C<Int32>\nstloc c\nldloca c\ninterface.borrow Read<Int32>\ncallvirt instance Read<Int32>::Get()\nret\n.end").unwrap();
    let p = LoadedProgram::new(&m).unwrap();
    assert!(p.verify().is_err());
    assert!(
        p.run(Limits::default())
            .unwrap_err()
            .message
            .contains("ambiguous explicit")
    );
}

#[test]
fn external_contract_mapping_preserves_module_identity_and_reference_requirements() {
    let contracts = ".module Contracts\n.interface Readable\n.method instance byref Read() -> Int32\n.end\n.end";
    let app = ".module App\n.references (Contracts)\n.entry Main\n.type C\n.implements [Contracts]Readable\n.method private instance byref Hidden() -> Int32\n.override instance [Contracts]Readable::Read()\nldc.i4 42\nret\n.end\n.end\n.function Main() -> Int32\n.local C c\nnewobj C\nstloc c\nldloca c\ninterface.borrow [Contracts]Readable\ncallvirt instance [Contracts]Readable::Read()\nret\n.end";
    let modules = neoclr::assembler::assemble_modules(&[app, contracts]).unwrap();
    let p = LoadedProgram::with_modules(
        &modules[0],
        neoclr::library::system().unwrap(),
        &modules[1..],
    )
    .unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    assert!(
        neoclr::assembler::assemble_modules(&[
            &app.replace(".references (Contracts)", ".references ()"),
            contracts
        ])
        .is_err()
    );
    assert!(
        neoclr::assembler::assemble_modules(&[
            &app.replace(
                ".override instance [Contracts]",
                ".override instance [Wrong]"
            ),
            contracts
        ])
        .is_err()
    );
}

#[test]
fn accessor_contracts_map_to_private_bodies_without_public_accessor_names() {
    let m = assemble(".module App\n.entry Main\n.interface Indexed\n.property instance Item(Int32) -> Int32\n.get instance Indexed::get_Item(Int32)\n.set instance Indexed::set_Item(Int32,Int32)\n.end\n.method instance byref get_Item(Int32 index) -> Int32\n.end\n.method instance byref set_Item(Int32 index,Int32 value) -> Void\n.end\n.end\n.type C\n.implements Indexed\n.field X Int32\n.method private instance byref Read(Int32 index) -> Int32\n.override instance Indexed::get_Item(Int32)\nldarg this\nldfld 0\nret\n.end\n.method private instance byref Write(Int32 index,Int32 value) -> Void\n.override instance Indexed::set_Item(Int32,Int32)\nldarg this\nldflda 0\nldarg value\nstobj Int32\nldvoid\nret\n.end\n.end\n.function Main() -> Int32\n.local C c\nldc.i4 0\nnewobj C\nstloc c\nldloca c\ninterface.borrow Indexed\nldc.i4 0\nldc.i4 42\ncallvirt instance Indexed::set_Item(Int32,Int32)\npop\nldloca c\ninterface.borrow Indexed\nldc.i4 0\ncallvirt instance Indexed::get_Item(Int32)\nret\n.end").unwrap();
    let p = LoadedProgram::new(&m).unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}
