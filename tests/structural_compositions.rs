use neoclr::{
    assemble,
    metadata::{Instruction, Type},
};
#[cfg(feature = "experimental-compositions")]
use neoclr::{Limits, LoadedProgram, Value};

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
    assert!(bad
        .run(Limits::default())
        .unwrap_err()
        .to_string()
        .contains("does not satisfy"));
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
        "arrayref<union<Left,Right>>&",
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
        "ldc.i4 1\nnewarr union<Left,Right>\nldc.i4 0\nldelema union<Left,Right>\npop",
        "ldc.i4 1\narray.alloc union<Left,Right>\npop",
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

#[cfg(feature = "experimental-compositions")]
#[test]
fn sealed_intersection_bounds_reject_before_normalization() {
    for ty in [
        "intersection<Both,Left>",
        "intersection<Both,Both>",
        "intersection<Left,union<Both,Right>>",
    ] {
        let source = format!(".module Bad\n.entry Main\n{TYPES}\n.function Main() -> Int32\n.local {ty} value\nldc.i4 0\nret\n.end");
        let source = source.replace(".type class Both", ".type class Both\n.sealed");
        assert!(assemble(&source).is_err(), "{ty}");
    }
    let source = format!(".module Good\n.entry Main\n{TYPES}\n.function Main() -> Int32\n.local union<Both,Left> value\nldc.i4 42\nret\n.end").replace(".type class Both", ".type class Both\n.sealed");
    assert!(assemble(&source).is_ok());
}

#[cfg(feature = "experimental-compositions")]
const GENERIC_TYPES: &str = r#"
.interface Read<T>
.method instance Get() -> T
.end
.end
.interface Marker<T>
.end
.type class Cell<T>
.implements Read<T>
.implements Marker<T>
.field Value T
.method instance .ctor(T value) -> noresult
ldarg this
ldarg value
stfld Cell<T>::Value
ret
.end
.method instance Get() -> T
ldarg this
ldfld Cell<T>::Value
ret
.end
.end
.type ValueCell<T>
.field Value T
.end
"#;

#[cfg(feature = "experimental-compositions")]
#[test]
fn closed_generic_intersection_preserves_arguments_and_interface_dispatch() {
    let source = format!(
        r#"
.module GenericCompositions
.entry Main
{GENERIC_TYPES}
.function Keep(intersection<Marker<Int32>,Read<Int32>> value) -> intersection<Read<Int32>,Marker<Int32>>
ldarg value
ret
.end
.function Main() -> Int32
.local intersection<Read<Int32>,Marker<Int32>> view
ldc.i4 42
newobj instance Cell<Int32>::.ctor(Int32)
stloc view
ldloc view
call Keep(intersection<Marker<Int32>,Read<Int32>>)
callvirt instance Read<Int32>::Get()
ret
.end
"#
    );
    let module = assemble(&source).unwrap();
    let p = LoadedProgram::new(&module).unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}

#[cfg(feature = "experimental-compositions")]
#[test]
fn generic_union_casts_use_constructed_identity() {
    for (target, succeeds) in [("Int32", true), ("Boolean", false)] {
        let source = format!(
            r#"
.module GenericUnion
.entry Main
{GENERIC_TYPES}
.function Main() -> Int32
.local union<Read<Int32>,Read<Boolean>> value
ldc.i4 42
newobj instance Cell<Int32>::.ctor(Int32)
stloc value
ldloc value
castclass intersection<Read<{target}>,Marker<{target}>>
pop
ldc.i4 42
ret
.end
"#
        );
        let module = assemble(&source).unwrap();
        let p = LoadedProgram::new(&module).unwrap();
        p.verify().unwrap();
        let result = p.run(Limits::default());
        if succeeds {
            assert_eq!(result.unwrap().value, Value::Int32(42));
        } else {
            assert!(result.is_err());
        }
    }
}

#[cfg(feature = "experimental-compositions")]
#[test]
fn generic_composition_rejects_invalid_arguments_and_sealed_bounds() {
    for ty in [
        "intersection<ValueCell<Int32>,Marker<Int32>>",
        "union<Read<Void>,Marker<Int32>>",
        "intersection<Read<Int32,Int32>,Marker<Int32>>",
    ] {
        let source = format!(".module InvalidGeneric\n.entry Main\n{GENERIC_TYPES}\n.function Main() -> Int32\n.local {ty} value\nldc.i4 42\nret\n.end");
        assert!(assemble(&source).is_err(), "{ty}");
    }
    for (ty, accepted) in [
        ("intersection<Cell<Int32>,Marker<Int32>>", false),
        ("intersection<Cell<Int32>,Cell<Int32>>", false),
        ("union<Cell<Int32>,Marker<Int32>>", true),
        ("union<Read<union<Read<Int32>,Marker<Int32>>>,Marker<Int32>>", true),
        ("intersection<Read<Cell<Int32>>,Marker<Cell<Int32>>>", true),
    ] {
        let types = GENERIC_TYPES.replace(".type class Cell<T>", ".type class Cell<T>\n.sealed");
        let source = format!(".module SealedGeneric\n.entry Main\n{types}\n.function Main() -> Int32\n.local {ty} value\nldc.i4 42\nret\n.end");
        assert_eq!(assemble(&source).is_ok(), accepted, "{ty}");
    }
}

#[cfg(feature = "experimental-compositions")]
#[test]
fn reference_wrapper_stores_and_returns_composition_arguments() {
    let source = format!(
        r#"
.module WrappedComposition
.entry Main
{TYPES}
.type class Response<T>
.field Value T
.end
.function Keep(Response<union<Left,Right>> value) -> Response<union<Right,Left>>
ldarg value
ret
.end
.function Main() -> Int32
.local Response<union<Left,Right>> response
ldc.i4 42
newobj Both
newobj Response<union<Left,Right>>
call Keep(Response<union<Right,Left>>)
stloc response
ldloc response
ldfld Response<union<Left,Right>>::Value
castclass Left
callvirt instance Left::Get()
ret
.end
"#
    );
    for kind in ["union", "intersection"] {
        let source = source.replace("union<", &format!("{kind}<"));
        let module = assemble(&source).unwrap();
        let module = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
        let p = LoadedProgram::new(&module).unwrap();
        p.verify().unwrap();
        assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    }
}

#[cfg(feature = "experimental-compositions")]
#[test]
fn nested_reference_wrapper_preserves_union_payload_across_collection() {
    let allocations = "newobj Neither\npop\n".repeat(8);
    let source = format!(
        r#"
.module NestedComposition
.entry Main
{TYPES}
.type class Response<T>
.field Value T
.end
.type class Envelope<T>
.field Value T
.end
.function Main() -> Int32
.local Envelope<union<Response<Int32>,Left>> result
ldc.i4 42
newobj Response<Int32>
newobj Envelope<union<Response<Int32>,Left>>
stloc result
{allocations}
ldloc result
ldfld Envelope<union<Response<Int32>,Left>>::Value
castclass Response<Int32>
ldfld Response<Int32>::Value
ret
.end
"#
    );
    let module = assemble(&source).unwrap();
    let p = LoadedProgram::new(&module).unwrap();
    p.verify().unwrap();
    let result = p
        .run(Limits {
            heap_objects: 3,
            ..Limits::default()
        })
        .unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert!(result.heap.collections() > 1);
    assert!(result.heap.reclaimed_objects() >= 8);
}

#[cfg(feature = "experimental-compositions")]
#[test]
fn composition_wrappers_reject_unsupported_storage_after_substitution() {
    for (kind, field) in [("class", "T&"), ("", "T")] {
        let source = format!(".module InvalidWrapper\n.entry Main\n{TYPES}\n.type {kind} Wrapper<T>\n.field Value {field}\n.end\n.function Main() -> Int32\n.local Wrapper<union<Left,Right>> value\nldc.i4 42\nret\n.end");
        assert!(assemble(&source).is_err(), "{kind} {field}");
    }
}

#[cfg(feature = "experimental-compositions")]
#[test]
fn composition_wrappers_preserve_invariance_and_validate_field_writes() {
    for body in [
        "ldc.i4 42\nnewobj Both\nnewobj Wrapper<union<Left,Right>>\ncall Need(Wrapper<intersection<Left,Right>>)\npop",
        "ldc.i4 42\nnewobj Both\nnewobj Wrapper<union<Left,Right>>\nnewobj Neither\nstfld Wrapper<union<Left,Right>>::Value",
    ] {
        let source = format!(".module InvalidUse\n.entry Main\n{TYPES}\n.type class Wrapper<T>\n.field Value T\n.end\n.function Need(Wrapper<intersection<Left,Right>> value) -> Int32\nldc.i4 42\nret\n.end\n.function Main() -> Int32\n{body}\nldc.i4 42\nret\n.end");
        let module = assemble(&source).unwrap();
        let p = LoadedProgram::new(&module).unwrap();
        assert!(p.verify().is_err(), "{body}");
        assert!(p.run(Limits::default()).is_err(), "{body}");
    }
}

#[test]
fn serialized_wrapper_composition_retains_feature_gate() {
    let mut module = assemble(&format!(
        ".module WrappedRaw\n.entry Main\n{TYPES}\n.type class Wrapper<T>\n.field Value T\n.end\n.function Main() -> Int32\nldc.i4 42\nret\n.end"
    )).unwrap();
    module
        .functions
        .iter_mut()
        .find(|f| f.name == "Main")
        .unwrap()
        .locals
        .push(Type::Constructed {
            definition: "Wrapper".into(),
            arguments: vec![Type::Union(vec![
                Type::from_name("Left"),
                Type::from_name("Right"),
            ])],
        });
    let result = neoclr::load(&serde_json::to_string(&module).unwrap());
    assert_eq!(
        result.is_ok(),
        cfg!(feature = "experimental-compositions"),
        "{result:?}"
    );
}

#[cfg(feature = "experimental-compositions")]
const WRAPPER_METHODS: &str = r#"
.type class Wrapper<T>
.field Value T
.method instance .ctor(T value) -> noresult
ldarg this
ldarg value
stfld Wrapper<T>::Value
ret
.end
.method instance Get() -> T
ldarg this
ldfld Wrapper<T>::Value
ret
.end
.method instance Set(T value) -> noresult
ldarg this
ldarg value
stfld Wrapper<T>::Value
ret
.end
.method instance Forward() -> T
ldarg this
call instance Wrapper<T>::Get()
ret
.end
.end
"#;

#[cfg(feature = "experimental-compositions")]
#[test]
fn composition_wrapper_constructor_and_instance_calls_preserve_payloads() {
    for kind in ["union", "intersection"] {
        let source = format!(
            r#"
.module WrapperCalls
.entry Main
{TYPES}
{WRAPPER_METHODS}
.function Main() -> Int32
.local Wrapper<{kind}<Left,Right>> wrapper
ldc.i4 7
newobj Both
newobj instance Wrapper<{kind}<Left,Right>>::.ctor({kind}<Right,Left>)
stloc wrapper
ldloc wrapper
ldc.i4 42
newobj Both
callvirt instance Wrapper<{kind}<Right,Left>>::Set({kind}<Left,Right>)
ldloc wrapper
callvirt instance Wrapper<{kind}<Left,Right>>::Forward()
castclass Left
callvirt instance Left::Get()
ret
.end
"#
        );
        let module = assemble(&source).unwrap();
        let module = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
        let p = LoadedProgram::new(&module).unwrap();
        p.verify().unwrap();
        assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    }
}

#[cfg(feature = "experimental-compositions")]
#[test]
fn wrapper_method_substitution_checks_transitive_bodies() {
    for operation in [
        ".local T temporary\nldloca temporary\npop\n",
        "ldtoken T\npop\n",
    ] {
        let methods = WRAPPER_METHODS.replace(
            ".method instance Get() -> T\n",
            &format!(".method instance Get() -> T\n{operation}"),
        );
        let source = format!(".module HiddenCompound\n.entry Main\n{TYPES}\n{methods}\n.function Main() -> Int32\nldc.i4 42\nnewobj Both\nnewobj instance Wrapper<union<Left,Right>>::.ctor(union<Left,Right>)\ncallvirt instance Wrapper<union<Left,Right>>::Forward()\npop\nldc.i4 42\nret\n.end");
        // The operation is legal for an ordinary reference argument; only closing
        // over a composition triggers the additional experimental admission rule.
        assert!(assemble(&source.replace("union<Left,Right>", "Both")).is_ok());
        let error = assemble(&source).unwrap_err().to_string();
        assert!(error.contains("compound"), "{operation}: {error}");
    }
}

#[cfg(feature = "experimental-compositions")]
#[test]
fn wrapper_calls_reject_wrong_receiver_payload_and_null() {
    for body in [
        "newobj Neither\nnewobj instance Wrapper<union<Left,Right>>::.ctor(union<Left,Right>)\npop",
        "ldc.i4 42\nnewobj Both\nnewobj instance Wrapper<union<Left,Right>>::.ctor(union<Left,Right>)\ncallvirt instance Wrapper<intersection<Left,Right>>::Get()\npop",
    ] {
        let source = format!(".module WrongCall\n.entry Main\n{TYPES}\n{WRAPPER_METHODS}\n.function Main() -> Int32\n{body}\nldc.i4 42\nret\n.end");
        let module = assemble(&source).unwrap();
        let p = LoadedProgram::new(&module).unwrap();
        assert!(p.verify().is_err());
        assert!(p.run(Limits::default()).is_err());
    }
    let source = format!(".module NullCall\n.entry Main\n{TYPES}\n{WRAPPER_METHODS}\n.function Main() -> Int32\nnewobj Neither\nisinst Wrapper<union<Left,Right>>\ncallvirt instance Wrapper<union<Left,Right>>::Forward()\npop\nldc.i4 42\nret\n.end");
    let module = assemble(&source).unwrap();
    let p = LoadedProgram::new(&module).unwrap();
    p.verify().unwrap();
    assert!(p
        .run(Limits::default())
        .unwrap_err()
        .to_string()
        .contains("null class receiver"));
}

#[cfg(feature = "experimental-compositions")]
#[test]
fn nested_wrapper_method_payload_survives_collection() {
    let allocations = "newobj Neither\npop\n".repeat(8);
    let source = format!(
        r#"
.module NestedWrapperCalls
.entry Main
{TYPES}
{WRAPPER_METHODS}
.function Main() -> Int32
.local Wrapper<union<Wrapper<Int32>,Left>> result
ldc.i4 42
newobj instance Wrapper<Int32>::.ctor(Int32)
newobj instance Wrapper<union<Wrapper<Int32>,Left>>::.ctor(union<Wrapper<Int32>,Left>)
stloc result
{allocations}
ldloc result
callvirt instance Wrapper<union<Wrapper<Int32>,Left>>::Forward()
castclass Wrapper<Int32>
callvirt instance Wrapper<Int32>::Get()
ret
.end
"#
    );
    let module = assemble(&source).unwrap();
    let p = LoadedProgram::new(&module).unwrap();
    p.verify().unwrap();
    let result = p
        .run(Limits {
            heap_objects: 3,
            ..Limits::default()
        })
        .unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert!(result.heap.collections() > 1);
    assert!(result.heap.reclaimed_objects() >= 8);
}

#[cfg(feature = "experimental-compositions")]
#[test]
fn wrapper_call_admission_deduplicates_recursive_specializations() {
    let methods = WRAPPER_METHODS.replace(
        "call instance Wrapper<T>::Get()",
        "call instance Wrapper<T>::Forward()",
    );
    let source = format!(".module RecursiveWrapper\n.entry Main\n{TYPES}\n{methods}\n.function Main() -> Int32\nldc.i4 42\nnewobj Both\nnewobj instance Wrapper<union<Left,Right>>::.ctor(union<Left,Right>)\ncallvirt instance Wrapper<union<Left,Right>>::Forward()\npop\nldc.i4 42\nret\n.end");
    let module = assemble(&source).unwrap();
    LoadedProgram::new(&module).unwrap().verify().unwrap();
    // This body recurses at execution; admission must finish without executing it.
}

#[cfg(feature = "experimental-compositions")]
#[test]
fn reference_composition_arrays_preserve_payloads_and_gc_roots() {
    for kind in ["union", "intersection"] {
        let allocations = "newobj Neither\npop\n".repeat(8);
        let body = format!(
            r#"
.local arrayref<{kind}<Left,Right>> values
ldc.i4 1
newarr {kind}<Left,Right>
stloc values
ldloc values
ldc.i4 0
ldelem {kind}<Right,Left>
ref.isnull
brtrue empty
fault "array default must be null"
empty:
ldloc values
ldc.i4 0
ldc.i4 42
newobj Both
stelem {kind}<Right,Left>
{allocations}
ldloc values
ldc.i4 0
ldelem {kind}<Left,Right>
castclass Left
callvirt instance Left::Get()
"#
        );
        let p = program(&body, "");
        p.verify().unwrap();
        let result = p
            .run(Limits {
                heap_objects: 3,
                ..Limits::default()
            })
            .unwrap();
        assert_eq!(result.value, Value::Int32(42));
        assert!(result.heap.collections() > 1);
        assert!(result.heap.reclaimed_objects() >= 8);
    }
}

#[cfg(feature = "experimental-compositions")]
#[test]
fn composition_array_writes_and_element_identity_are_checked() {
    for operation in [
        "newobj Neither\nstelem union<Left,Right>",
        "ldelem intersection<Left,Right>\npop",
        "ldelem Left\npop",
    ] {
        let body = format!("ldc.i4 1\nnewarr union<Left,Right>\nldc.i4 0\n{operation}\nldc.i4 42");
        let p = program(&body, "");
        assert!(p.verify().is_err(), "{operation}");
        assert!(p.run(Limits::default()).is_err(), "{operation}");
    }
    let p = program(
        "ldc.i4 1\nnewarr union<Left,Right>\ncall Need(arrayref<intersection<Left,Right>>) ",
        ".function Need(arrayref<intersection<Left,Right>> values) -> Int32\nldc.i4 42\nret\n.end",
    );
    assert!(p.verify().is_err());
    assert!(p.run(Limits::default()).is_err());
}

#[cfg(feature = "experimental-compositions")]
#[test]
fn substituted_array_storage_uses_checked_reference_vectors() {
    let methods = r#"
.type class Buffer<T>
.field Values arrayref<T>
.method instance .ctor(T value) -> noresult
ldarg this
ldc.i4 1
array.reserve T
stfld Buffer<T>::Values
ldarg this
ldfld Buffer<T>::Values
ldc.i4 0
ldarg value
stelem T
ret
.end
.method instance Get() -> T
ldarg this
ldfld Buffer<T>::Values
ldc.i4 0
ldelem T
ret
.end
.end
"#;
    for kind in ["union", "intersection"] {
        let p = program(
            &format!(
                "ldc.i4 42\nnewobj Both\nnewobj instance Buffer<{kind}<Left,Right>>::.ctor({kind}<Left,Right>)\ncall instance Buffer<{kind}<Left,Right>>::Get()\ncastclass Left\ncallvirt instance Left::Get()"
            ),
            methods,
        );
        p.verify().unwrap();
        assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    }
}

#[cfg(feature = "experimental-compositions")]
#[test]
fn reserved_composition_array_slots_require_initialization_and_bounds() {
    for (body, expected) in [
        (
            "ldc.i4 1\narray.reserve union<Left,Right>\nldc.i4 0\nldelem union<Left,Right>\npop\nldc.i4 42",
            "uninitialized",
        ),
        (
            "ldc.i4 1\nnewarr union<Left,Right>\nldc.i4 1\nldelem union<Left,Right>\npop\nldc.i4 42",
            "index",
        ),
    ] {
        let p = program(body, "");
        p.verify().unwrap();
        let error = p.run(Limits::default()).unwrap_err().to_string();
        assert!(error.contains(expected), "{error}");
    }
}

#[test]
fn serialized_composition_array_retains_feature_gate() {
    let mut module = assemble(&format!(
        ".module ArrayRaw\n.entry Main\n{TYPES}\n.function Main() -> Int32\nldc.i4 42\nret\n.end"
    ))
    .unwrap();
    module
        .functions
        .iter_mut()
        .find(|f| f.name == "Main")
        .unwrap()
        .locals
        .push(Type::ArrayRef(Box::new(Type::Union(vec![
            Type::from_name("Left"),
            Type::from_name("Right"),
        ]))));
    let result = neoclr::load(&serde_json::to_string(&module).unwrap());
    assert_eq!(
        result.is_ok(),
        cfg!(feature = "experimental-compositions"),
        "{result:?}"
    );
}

#[cfg(feature = "experimental-compositions")]
const COMPOSITION_INTERFACE_WRAPPER: &str = r#"
.interface Reader<T>
.method instance Get() -> T
.end
.end
.interface DerivedReader<T>
.implements Reader<T>
.end
.type class Cell<T>
.implements DerivedReader<T>
.field Value T
.method instance Get() -> T
ldarg this
ldfld Cell<T>::Value
ret
.end
.end
"#;

#[cfg(feature = "experimental-compositions")]
#[test]
fn composition_arguments_preserve_inherited_interface_identity() {
    for kind in ["union", "intersection"] {
        let body = format!(
            r#"
.local Reader<{kind}<Left,Right>> reader
ldc.i4 42
newobj Both
newobj Cell<{kind}<Left,Right>>
castclass DerivedReader<{kind}<Right,Left>>
stloc reader
ldloc reader
castclass Cell<{kind}<Right,Left>>
call instance Cell<{kind}<Left,Right>>::Get()
castclass Left
callvirt instance Left::Get()
"#
        );
        let p = program(&body, COMPOSITION_INTERFACE_WRAPPER);
        p.verify().unwrap();
        assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    }
}

#[cfg(feature = "experimental-compositions")]
#[test]
fn composition_interface_arguments_remain_invariant() {
    let make = "ldc.i4 42\nnewobj Both\nnewobj Cell<union<Left,Right>>";
    for target in ["Reader<intersection<Left,Right>>", "Reader<Left>"] {
        let p = program(
            &format!("{make}\ncastclass {target}\npop\nldc.i4 42"),
            COMPOSITION_INTERFACE_WRAPPER,
        );
        p.verify().unwrap();
        assert!(p.run(Limits::default()).is_err(), "{target}");
        let p = program(
            &format!(".local {target} reader\n{make}\nstloc reader\nldc.i4 42"),
            COMPOSITION_INTERFACE_WRAPPER,
        );
        assert!(p.verify().is_err(), "{target}");
        assert!(p.run(Limits::default()).is_err(), "{target}");
    }
}

#[cfg(feature = "experimental-compositions")]
#[test]
fn composition_argument_interface_calls_remain_gated() {
    let source = format!(
        ".module InterfaceCall\n.entry Main\n{TYPES}\n{COMPOSITION_INTERFACE_WRAPPER}\n.function Main() -> Int32\nldc.i4 42\nnewobj Both\nnewobj Cell<union<Left,Right>>\ncallvirt instance Reader<union<Left,Right>>::Get()\npop\nldc.i4 42\nret\n.end"
    );
    let error = assemble(&source).unwrap_err().to_string();
    assert!(error.contains("interface dispatch"), "{error}");
}
