use neoclr::{Limits, LoadedProgram, Value, assembler::assemble_modules, metadata_container};

const BASE: &str = ".module Models\n.revision r1\n.type class abstract Base\n.field Value Int32\n.method protected instance .ctor(Int32 value) -> noresult\nldarg this\nldarg value\nstfld Base::Value\nret\n.end\n.end";
const CHILD: &str = ".module Children\n.references (Models#r1)\n.type class Child\n.extends Base\n.method instance .ctor() -> noresult\nldarg this\nldc.i4 42\ncall instance Base::.ctor(Int32)\nret\n.end\n.end";
const APP: &str = ".module App\n.references (Models#r1,Children)\n.entry Main\n.function Main() -> Int32\nnewobj instance Child::.ctor()\nldfld Base::Value\nret\n.end";

#[test]
fn protected_base_constructor_executes_across_binary_assemblies() {
    let modules = assemble_modules(&[APP, BASE, CHILD]).unwrap();
    let decoded: Vec<_> = modules
        .iter()
        .map(|m| {
            metadata_container::decode_envelope(&metadata_container::write_module(m).unwrap())
                .unwrap()
        })
        .collect();
    let program = LoadedProgram::with_modules(
        &decoded[0],
        neoclr::library::system().unwrap(),
        &decoded[1..],
    )
    .unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn unrelated_caller_cannot_invoke_protected_constructor() {
    let bad = APP.replace(
        "newobj instance Child::.ctor()\nldfld Base::Value",
        "ldc.i4 42\nnewobj instance Base::.ctor(Int32)\nldfld Base::Value",
    );
    let base = BASE.replace("class abstract", "class");
    let error = assemble_modules(&[&bad, &base, CHILD]).unwrap_err();
    assert!(error.message.contains("method access denied"), "{error}");
}

#[test]
fn unsupported_protected_members_reject_explicitly() {
    let bad = BASE.replace(".ctor(Int32 value)", "Other(Int32 value)");
    let error = assemble_modules(&[&bad]).unwrap_err();
    assert!(
        error
            .message
            .contains("protected visibility currently requires"),
        "{error}"
    );
}

#[test]
fn closed_family_admits_local_children_but_rejects_external_direct_children() {
    let root = BASE.replace(".field Value", ".closedhierarchy\n.field Value");
    let error = assemble_modules(&[APP, &root, CHILD]).unwrap_err();
    assert!(error.message.contains("closed hierarchy"), "{error}");
    let local = format!(
        "{root}\n{}",
        CHILD
            .split(".type")
            .nth(1)
            .map(|s| format!(".type{s}"))
            .unwrap()
    );
    let app = APP.replace("Models#r1,Children", "Models#r1");
    let modules = assemble_modules(&[&app, &local]).unwrap();
    let program = LoadedProgram::with_modules(
        &modules[0],
        neoclr::library::system().unwrap(),
        &modules[1..],
    )
    .unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn open_local_branch_of_closed_family_remains_extensible() {
    let root = BASE.replace(".field Value", ".closedhierarchy\n.field Value");
    let child_definition = CHILD.split_once(".type").unwrap().1;
    let local = format!("{root}\n.type{child_definition}");
    let leaf = ".module Leaves\n.references (Models#r1)\n.type class Leaf\n.extends Child\n.method instance .ctor() -> noresult\nldarg this\ncall instance Child::.ctor()\nret\n.end\n.end";
    let app = APP
        .replace("Models#r1,Children", "Models#r1,Leaves")
        .replace("newobj instance Child", "newobj instance Leaf");
    let modules = assemble_modules(&[&app, &local, leaf]).unwrap();
    let program = LoadedProgram::with_modules(
        &modules[0],
        neoclr::library::system().unwrap(),
        &modules[1..],
    )
    .unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn closed_class_flags_cannot_describe_an_instantiable_or_sealed_root() {
    for root in [
        BASE.replace("class abstract", "class")
            .replace(".field Value", ".closedhierarchy\n.field Value"),
        BASE.replace(".field Value", ".closedhierarchy\n.sealed\n.field Value"),
    ] {
        let error = assemble_modules(&[&root]).unwrap_err();
        assert!(
            error.message.contains("closed class hierarchy requires"),
            "{error}"
        );
    }
}

#[test]
fn closed_interfaces_check_direct_ownership_but_allow_open_branches() {
    let root = ".module Contracts\n.revision r1\n.interface Closed\n.closedhierarchy\n.end\n.interface Open\n.implements Closed\n.end\n.type Local\n.implements Closed\n.end";
    let child = ".module Children\n.references (Contracts#r1)\n.type Child\n.implements Open\n.end";
    assemble_modules(&[root, child]).unwrap();
    for declaration in [".type Child", ".interface Child"] {
        let bad = child
            .replace(".type Child", declaration)
            .replace(".implements Open", ".implements Closed");
        let error = assemble_modules(&[root, &bad]).unwrap_err();
        assert!(
            error.message.contains("closed interface hierarchy"),
            "{error}"
        );
    }
    // Same module name with a different revision does not grant ownership.
    let old = child
        .replace(".module Children", ".module Contracts\n.revision r0")
        .replace(".implements Open", ".implements Closed");
    assert!(assemble_modules(&[root, &old]).is_err());
}
