use neoclr::assemble;
#[cfg(feature = "experimental-compositions")]
use neoclr::{Limits, LoadedProgram, Value, assembler::parse_type, load};

const TYPES: &str = r#"
.interface A
.end
.interface B
.end
.type class Both
.sealed
.implements A
.implements B
.end
.type class OnlyA
.implements A
.end
.type class OnlyB
.implements B
.end
.type class Neither
.end
.type class Box<T>
.end
"#;

#[cfg(not(feature = "experimental-compositions"))]
#[test]
fn structural_constraints_require_explicit_runtime_feature() {
    let source =
        format!(".module Bounds\n{TYPES}\n.type class Either<T>\n.constraint T union<A,B>\n.end");
    assert!(assemble(&source).is_err());
}

#[cfg(feature = "experimental-compositions")]
fn constrained_module(bound: &str) -> neoclr::Module {
    let source = format!(
        ".module Bounds\n{TYPES}\n.type class Restricted<T>\n.constraint T {bound}\n.field Value T\n.end"
    );
    let module = assemble(&source).unwrap();
    load(&serde_json::to_string(&module).unwrap()).unwrap()
}

#[cfg(feature = "experimental-compositions")]
#[test]
fn native_type_bounds_preserve_any_of_all_of_and_closed_satisfiers() {
    for (bound, good, bad) in [
        (
            "union<A,B>",
            vec!["OnlyA", "OnlyB", "Both", "union<OnlyA,OnlyB>"],
            vec!["Neither", "union<OnlyA,Neither>", "Int32"],
        ),
        (
            "intersection<A,B>",
            vec!["Both", "intersection<A,B>"],
            vec!["OnlyA", "OnlyB", "union<OnlyA,OnlyB>"],
        ),
        (
            "union<Box<OnlyA>,B>",
            vec!["Box<OnlyA>", "OnlyB"],
            vec!["Box<OnlyB>", "OnlyA"],
        ),
    ] {
        let module = constrained_module(bound);
        for argument in good {
            assert!(
                module
                    .instantiated_fields(&parse_type(&format!("Restricted<{argument}>")).unwrap())
                    .is_ok(),
                "{argument}: {bound}"
            );
        }
        for argument in bad {
            let error = module
                .instantiated_fields(&parse_type(&format!("Restricted<{argument}>")).unwrap())
                .unwrap_err();
            assert!(error.message.contains("violates"), "{argument}: {error}");
        }
        let json = serde_json::to_value(&module).unwrap();
        assert!(json.to_string().contains(if bound.starts_with("union") {
            "Union"
        } else {
            "Intersection"
        }));
    }
}

#[cfg(feature = "experimental-compositions")]
#[test]
fn nominal_method_arguments_are_checked_even_without_verification() {
    for (bound, argument, valid) in [
        ("union<A,B>", "OnlyA", true),
        ("union<A,B>", "OnlyB", true),
        ("union<A,B>", "Neither", false),
        ("intersection<A,B>", "Both", true),
        ("intersection<A,B>", "OnlyA", false),
    ] {
        // Construct a valid definition first so failures must arise at runtime
        // resolution of the requested specialization, not only source checking.
        let source = format!(
            ".module Methods\n{TYPES}\n.function Check<T>() -> Int32\n.constraint T {bound}\nldc.i4 42\nret\n.end"
        );
        let module = assemble(&source).unwrap();
        let program = LoadedProgram::new(&module).unwrap();
        let target =
            neoclr::assembler::parse_function_ref(&format!("Check<{argument}>()")).unwrap();
        let result = program
            .resolve_function(&target)
            .and_then(|function| function.invoke(vec![], Limits::default()));
        if valid {
            assert_eq!(result.unwrap().value, Value::Int32(42));
        } else {
            assert!(result.unwrap_err().message.contains("violates"));
        }
    }
}

#[cfg(feature = "experimental-compositions")]
#[test]
fn invalid_structural_bounds_cannot_normalize_away() {
    for bound in [
        "intersection<Both,A>",
        "intersection<Both,Both>",
        "union<Int32,A>",
        "union<Unknown,A>",
        "union<Box<T>,A>",
    ] {
        let source = format!(
            ".module Invalid\n{TYPES}\n.type class Restricted<T>\n.constraint T {bound}\n.end"
        );
        assert!(assemble(&source).is_err(), "{bound}");
    }
}

#[cfg(feature = "experimental-compositions")]
#[test]
fn documented_native_constraint_consumer_executes() {
    let module = assemble(include_str!(
        "../docs/experiments/composition-constraints/native.neil"
    ))
    .unwrap();
    let module = load(&serde_json::to_string(&module).unwrap()).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}
