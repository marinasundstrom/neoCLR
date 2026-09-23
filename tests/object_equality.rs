use neoclr::{FaultCode, Limits, LoadedProgram, Module, Value};
use std::{process::Command, sync::OnceLock};

fn library() -> &'static Module {
    static LIBRARY: OnceLock<Module> = OnceLock::new();
    LIBRARY.get_or_init(|| {
        let output = Command::new("python3")
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .args(["-c", "import runpy; m=runpy.run_path('docs/experiments/raven-target/collection_library.py'); print(m['build'](m['ROOT'] / 'runtime/System.neoil'))"])
            .output().unwrap();
        assert!(output.status.success());
        neoclr::assemble(std::str::from_utf8(&output.stdout).unwrap()).unwrap()
    })
}
fn run(
    body: &str,
    declarations: &str,
    returns: &str,
    heap_objects: usize,
) -> Result<neoclr::Execution, neoclr::Fault> {
    let source = format!(
        ".module Equality\n.entry Main\n{declarations}\n.function Main() -> {returns}\n{body}\nret\n.end"
    );
    let app = neoclr::assembler::read_modules(
        &[neoclr::assembler::ModuleInput::Source(&source)],
        library(),
    )?
    .remove(0);
    let app: Module = serde_json::from_str(&serde_json::to_string(&app).unwrap()).unwrap();
    let program = LoadedProgram::with_library(&app, library())?;
    program.verify()?;
    program.run(Limits {
        heap_objects,
        ..Limits::default()
    })
}
const CELL: &str = ".type class Cell\n.field Number Int32\n.end";
const IDENTITY: &str = "call System.Object::ReferenceEquals(System.Object,System.Object)";
const EQUALS: &str = "callvirt instance System.Object::Equals(System.Object)";
const HASH: &str = "callvirt instance System.Object::GetHashCode()";

#[test]
fn class_default_equality_and_reference_identity_agree() {
    for call in [IDENTITY, EQUALS] {
        for (right, expected) in [("ldloc first", true), ("ldc.i4 1\nnewobj Cell", false)] {
            let body = format!(
                ".local Cell first\nldc.i4 1\nnewobj Cell\nstloc first\nldloc first\n{right}\n{call}"
            );
            assert_eq!(
                run(&body, CELL, "Boolean", 8).unwrap().value,
                Value::Boolean(expected)
            );
        }
    }
}

#[test]
fn identity_hash_survives_mutation_object_views_and_collection() {
    let mut body = format!(
        ".local Cell first\n.local System.Object alias\n.local Int32 hash\nldc.i4 1\nnewobj Cell\nstloc first\nldloc first\n{HASH}\nstloc hash\nldloc first\ncastclass System.Object\nstloc alias\nldloc first\nldc.i4 9\nstfld Cell::Number\n"
    );
    for _ in 0..8 {
        body.push_str("ldc.i4 0\nnewobj Cell\npop\n");
    }
    body.push_str(&format!("ldloc alias\n{HASH}\nldloc hash\nceq"));
    let result = run(&body, CELL, "Boolean", 2).unwrap();
    assert_eq!(result.value, Value::Boolean(true));
    assert!(result.heap.collections() > 1);
}

#[test]
fn reference_identity_handles_nulls_and_separate_boxes() {
    for (left, right, expected) in [
        ("ldloc empty", "ldloc empty", true),
        ("ldloc box", "ldloc empty", false),
        ("ldloc empty", "ldloc box", false),
        ("ldloc box", "ldloc box", true),
        ("ldloc box", "ldc.i4 42\nbox Int32", false),
    ] {
        let body = format!(
            ".local System.Object empty\n.local System.Object box\nldloca empty\ninitobj System.Object\nldc.i4 42\nbox Int32\nstloc box\n{left}\n{right}\n{IDENTITY}"
        );
        assert_eq!(
            run(&body, "", "Boolean", 8).unwrap().value,
            Value::Boolean(expected)
        );
    }
}

#[test]
fn class_instance_null_contract_is_enforced_for_direct_and_virtual_calls() {
    for call in ["call", "callvirt"] {
        for (member, args, returns) in [
            ("GetHashCode()", "", "Int32"),
            ("Equals(System.Object)", "ldloc empty\n", "Boolean"),
        ] {
            let body = format!(
                ".local System.Object empty\nldloca empty\ninitobj System.Object\nldloc empty\n{args}{call} instance System.Object::{member}"
            );
            assert_eq!(
                run(&body, "", returns, 8).unwrap_err().code,
                FaultCode::NullReference
            );
        }
    }
    let body = format!(
        ".local System.Object empty\nldloca empty\ninitobj System.Object\nldc.i4 1\nnewobj Cell\nldloc empty\n{EQUALS}"
    );
    assert_eq!(
        run(&body, CELL, "Boolean", 8).unwrap().value,
        Value::Boolean(false)
    );
}

#[test]
fn overrides_do_not_replace_reference_identity_or_explicit_base_equality() {
    let declarations = ".type class Key\n.extends System.Object\n.method instance override Equals(System.Object other) -> Boolean\nldc.bool true\nret\n.end\n.method instance override GetHashCode() -> Int32\nldc.i4 42\nret\n.end\n.end";
    for (call, expected) in [
        (EQUALS, true),
        (IDENTITY, false),
        ("call instance System.Object::Equals(System.Object)", false),
    ] {
        let body = format!("newobj Key\nnewobj Key\n{call}");
        assert_eq!(
            run(&body, declarations, "Boolean", 8).unwrap().value,
            Value::Boolean(expected)
        );
    }
    assert_eq!(
        run(&format!("newobj Key\n{HASH}"), declarations, "Int32", 8)
            .unwrap()
            .value,
        Value::Int32(42)
    );
}

#[test]
fn arrays_use_identity_equality_and_stable_default_hash() {
    for (right, expected) in [("ldloc array", true), ("ldc.i4 0\nnewarr Int32", false)] {
        let body = format!(
            ".local arrayref<Int32> array\nldc.i4 0\nnewarr Int32\nstloc array\nldloc array\n{right}\n{EQUALS}"
        );
        assert_eq!(
            run(&body, "", "Boolean", 8).unwrap().value,
            Value::Boolean(expected)
        );
    }
    let body = format!(
        ".local arrayref<Int32> array\nldc.i4 0\nnewarr Int32\nstloc array\nldloc array\n{HASH}\nldloc array\ncastclass System.Object\n{HASH}\nceq"
    );
    assert_eq!(
        run(&body, "", "Boolean", 8).unwrap().value,
        Value::Boolean(true)
    );
}

#[test]
fn string_wrappers_cannot_accidentally_acquire_a_public_identity_contract() {
    for (body, returns) in [
        (
            format!("ldstr \"text\"\ncastclass System.Object\ndup\n{IDENTITY}"),
            "Boolean",
        ),
        (
            "ldstr \"text\"\ncastclass System.Object\ncall instance System.Object::GetHashCode()"
                .into(),
            "Int32",
        ),
    ] {
        let fault = run(&body, "", returns, 8).unwrap_err();
        assert!(
            fault
                .message
                .contains("String object identity is not supported"),
            "{fault}"
        );
    }
}

#[test]
fn boxed_virtual_value_equality_remains_explicitly_unsupported() {
    for (tail, returns) in [
        (format!("ldc.i4 42\nbox Int32\n{EQUALS}"), "Boolean"),
        (HASH.into(), "Int32"),
    ] {
        assert!(run(&format!("ldc.i4 42\nbox Int32\n{tail}"), "", returns, 8).is_err());
    }
}

#[test]
fn identity_imports_require_exact_signatures_and_report_managed_heap_service() {
    use neoclr::{RuntimeService, assemble, assembler::parse_function_ref};
    for (name, parameters, returns) in [
        (
            "ObjectReferenceEquals",
            "System.Object,System.Object",
            "Boolean",
        ),
        ("ObjectEquals", "System.Object,System.Object", "Boolean"),
        ("ObjectIdentityHash", "System.Object", "Int32"),
    ] {
        let source = format!(
            ".module Identity\n.type class abstract System.Object\n.end\n.function neoCLR.Runtime.{name}({parameters}) -> {returns}\n.methodimpl InternalCall\n.end"
        );
        let module = assemble(&source).unwrap();
        let program = LoadedProgram::new(&module).unwrap();
        program.verify().unwrap();
        let graph = program
            .analyze_reachability(
                &[parse_function_ref(&format!("neoCLR.Runtime.{name}({parameters})")).unwrap()],
                1,
            )
            .unwrap();
        assert_eq!(graph.required_services(), [RuntimeService::ManagedHeap]);
        let wrong = source.replace(&format!(") -> {returns}"), ") -> String");
        let fault = match assemble(&wrong) {
            Err(fault) => fault,
            Ok(module) => LoadedProgram::new(&module)
                .and_then(|program| program.verify())
                .unwrap_err(),
        };
        assert!(
            fault
                .message
                .contains("runtime binding return type mismatch"),
            "{fault}"
        );
    }
}
