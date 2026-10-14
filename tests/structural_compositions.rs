#[cfg(feature = "experimental-compositions")]
use neoclr::{Limits, LoadedProgram, Value};
use neoclr::{
    assemble,
    metadata::{Instruction, Type},
};

const TYPES: &str = "\
.type class abstract System.Object\n.end\n\
.interface Left\n.method instance Get() -> Int32\n.end\n.end\n\
.interface Right\n.end\n\
.type class Both\n.implements Left\n.implements Right\n.field Number Int32\n\
.method instance Get() -> Int32\nldarg this\nldfld Both::Number\nret\n.end\n.end\n\
.type class LeftOnly\n.implements Left\n\
.method instance Get() -> Int32\nldc.i4 1\nret\n.end\n.end\n\
.type class Neither\n.end\n.type ValueCell\n.end\n";

#[cfg(feature = "experimental-compositions")]
fn program(body: &str, helpers: &str) -> LoadedProgram {
    let source = format!(
        ".module Compound\n.entry Main\n{TYPES}\n{helpers}\n.function Main() -> Int32\n{body}\nret\n.end"
    );
    let module = assemble(&source).unwrap();
    let module = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    LoadedProgram::new(&module).unwrap()
}

#[cfg(feature = "experimental-compositions")]
#[test]
fn intersection_argument_return_and_local_preserve_dispatch() {
    let p = program(
        ".local intersection<Left,Right> view\nldc.i4 42\nnewobj Both\ncall Keep(intersection<Right,Left>)\nstloc view\nldloc view\ncallvirt instance Left::Get()",
        ".function Keep(intersection<Left,Right> value) -> intersection<Right,Left>\nldarg value\nret\n.end",
    );
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}

#[cfg(feature = "experimental-compositions")]
#[test]
fn union_requires_narrowing_and_checks_concrete_object() {
    let p = program(
        ".local union<Left,Right> value\nldc.i4 42\nnewobj Both\nstloc value\nldloc value\ncastclass intersection<Left,Right>\ncastclass Left\ncallvirt instance Left::Get()",
        "",
    );
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    let bad = program(
        ".local union<Left,Right> value\nnewobj LeftOnly\nstloc value\nldloc value\ncastclass intersection<Left,Right>\npop\nldc.i4 0",
        "",
    );
    bad.verify().unwrap();
    assert!(
        bad.run(Limits::default())
            .unwrap_err()
            .to_string()
            .contains("does not satisfy")
    );
    let bad_static = program(
        ".local union<Left,Right> value\nnewobj LeftOnly\nstloc value\nldloc value\ncallvirt instance Left::Get()",
        "",
    );
    assert!(bad_static.verify().is_err());
    assert!(bad_static.run(Limits::default()).is_err());
}

#[cfg(feature = "experimental-compositions")]
#[test]
fn compound_null_casts_and_type_tests_follow_reference_behavior() {
    let p = program(
        ".local union<Left,Right> value\nnewobj Neither\nisinst union<Left,Right>\nstloc value\nldloc value\ncastclass intersection<Left,Right>\nisinst union<Right,Left>\nref.isnull\nbrfalse bad\nldc.i4 42\nret\nbad:\nldc.i4 0",
        "",
    );
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    let p = program(
        "newobj Neither\nisinst intersection<Left,Right>\npop\nldc.i4 42",
        "",
    );
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}

#[cfg(feature = "experimental-compositions")]
#[test]
fn canonicalization_happens_before_call_binding() {
    let p = program(
        "ldc.i4 42\nnewobj Both\ncall Read(intersection<Left,intersection<Right,Left>>)\n",
        ".function Read(intersection<Right,Left> value) -> Int32\nldarg value\ncastclass Left\ncallvirt instance Left::Get()\nret\n.end",
    );
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    let p = program(
        "ldc.i4 42\nnewobj Both\ncall Read(union<Both,Both>)",
        ".function Read(Both value) -> Int32\nldarg value\ncallvirt instance Left::Get()\nret\n.end",
    );
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}

#[cfg(feature = "experimental-compositions")]
#[test]
fn unsupported_shapes_and_address_operations_fail_admission() {
    for ty in [
        "union<Int32,Left>",
        "intersection<ValueCell,Left>",
        "union<ValueCell,ValueCell>",
        "union<Missing,Left>",
        "union<Left>",
        "union<Left,Right>&",
        "arrayref<union<Left,Right>>",
        "fn<union<Left,Right>,Int32>",
    ] {
        let source = format!(
            ".module Bad\n.entry Main\n{TYPES}\n.function Main() -> Int32\n.local {ty} value\nldc.i4 0\nret\n.end"
        );
        assert!(assemble(&source).is_err(), "{ty}");
    }
    for body in [
        ".local union<Left,Right> value\nldloca value\npop",
        "ldtoken union<Left,Right>\npop",
        "sizeof union<Left,Right>\npop",
    ] {
        let source = format!(
            ".module Bad\n.entry Main\n{TYPES}\n.function Main() -> Int32\n{body}\nldc.i4 0\nret\n.end"
        );
        assert!(assemble(&source).is_err(), "{body}");
    }
    let source = format!(
        ".module Bad\n.entry Main\n{TYPES}\n.type class Holder\n.field Value union<Left,Right>\n.end\n.function Main() -> Int32\nldc.i4 0\nret\n.end"
    );
    assert!(assemble(&source).is_err());
}

#[test]
fn serialized_compound_feature_boundary_cannot_be_bypassed_by_parser_omission() {
    let mut module = assemble(&format!(
        ".module Raw\n.entry Main\n{TYPES}\n.function Main() -> Int32\nldc.i4 0\nret\n.end"
    ))
    .unwrap();
    let main = module
        .functions
        .iter_mut()
        .find(|f| f.name == "Main")
        .unwrap();
    let ty = Type::Union(vec![Type::from_name("Left"), Type::from_name("Right")]);
    main.locals = vec![ty.clone()];
    if cfg!(feature = "experimental-compositions") {
        main.body = vec![
            Instruction::LoadTypeToken(ty),
            Instruction::Pop,
            Instruction::Int(0),
            Instruction::Return,
        ];
    }
    let error = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap_err();
    let expected = if cfg!(feature = "experimental-compositions") {
        "compound instruction operand"
    } else {
        "experimental-compositions"
    };
    assert!(error.to_string().contains(expected), "{error}");
}

#[cfg(feature = "experimental-compositions")]
#[test]
fn compound_local_retains_allocation_across_collection() {
    let mut body =
        String::from(".local intersection<Left,Right> view\nldc.i4 42\nnewobj Both\nstloc view\n");
    for _ in 0..8 {
        body.push_str("newobj Neither\npop\n");
    }
    body.push_str("ldloc view\ncallvirt instance Left::Get()");
    let p = program(&body, "");
    p.verify().unwrap();
    let result = p
        .run(Limits {
            heap_objects: 2,
            ..Limits::default()
        })
        .unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert!(result.heap.collections() > 1);
    assert!(result.heap.reclaimed_objects() >= 8);
}

#[cfg(feature = "experimental-compositions")]
#[test]
fn boxed_values_do_not_enter_reference_only_compositions_through_object() {
    let p = program(
        "ldc.i4 42\nbox Int32\nisinst union<System.Object,Left>\nref.isnull\nbrfalse bad\nldc.i4 42\nret\nbad:\nldc.i4 0",
        "",
    );
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    let p = program(
        "ldc.i4 42\nbox Int32\ncastclass union<System.Object,Left>\npop\nldc.i4 0",
        "",
    );
    p.verify().unwrap();
    assert!(p.run(Limits::default()).is_err());
    // Static Object membership cannot smuggle a boxed allocation through storage.
    let p = program(
        ".local union<System.Object,Left> value\nldc.i4 42\nbox Int32\nstloc value\nldc.i4 0",
        "",
    );
    p.verify().unwrap();
    assert!(p.run(Limits::default()).is_err());
}

#[cfg(feature = "experimental-compositions")]
#[test]
fn every_constituent_obeys_module_reference_and_access_checks() {
    use neoclr::assembler::assemble_modules;
    let contracts = ".module Contracts\n.interface Left\n.end\n.interface Right\n.end";
    let source = |references| {
        format!(
            ".module App\n.references ({references})\n.entry Main\n.function Main() -> Int32\n.local union<Left,Right> view\nldc.i4 42\nret\n.end"
        )
    };
    let error = assemble_modules(&[&source(""), contracts]).unwrap_err();
    assert!(error.to_string().contains("does not reference"), "{error}");
    let mut modules = assemble_modules(&[&source("Contracts"), contracts]).unwrap();
    let p = LoadedProgram::with_modules(
        &modules[0],
        neoclr::library::system().unwrap(),
        &modules[1..],
    )
    .unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    modules[1].types[0].visibility = neoclr::metadata::Visibility::Internal;
    let error = LoadedProgram::with_modules(
        &modules[0],
        neoclr::library::system().unwrap(),
        &modules[1..],
    )
    .unwrap_err();
    assert!(error.to_string().contains("access denied"), "{error}");
}

#[cfg(feature = "experimental-compositions")]
#[test]
fn saved_consumer_roundtrips_native_envelope_and_executes() {
    let module = assemble(include_str!(
        "../docs/experiments/structural-compositions/reference-compositions.neoil"
    ))
    .unwrap();
    let image = neoclr::metadata_container::write_module(&module).unwrap();
    let decoded = neoclr::metadata_container::load_envelope(&image).unwrap();
    let program = LoadedProgram::new(&decoded).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}
