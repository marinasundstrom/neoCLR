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
fn other_boxed_virtual_value_equality_remains_explicitly_unsupported() {
    for (tail, returns) in [
        (format!("ldc.i4 42\nbox Int32\n{EQUALS}"), "Boolean"),
        (HASH.into(), "Int32"),
    ] {
        assert!(run(&format!("ldc.i4 42\nnewobj Unimplemented\nbox Unimplemented\n{tail}"), ".type Unimplemented\n.field Number Int32\n.end", returns, 8).is_err());
    }
}

#[test]
fn identity_imports_require_exact_signatures_and_report_managed_heap_service() {
    use neoclr::{assemble, assembler::parse_function_ref, RuntimeService};
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

#[test]
fn boxed_int32_virtual_equality_uses_exact_type_and_value() {
    for (right, expected) in [
        ("ldc.i4 42\nbox Int32", true),
        ("ldc.i4 7\nbox Int32", false),
        ("ldc.i8 42\nbox Int64", false),
        ("ldc.bool true\nbox Boolean", false),
        ("ldstr \"42\"\ncastclass System.Object", false),
        ("ldc.i4 42\nnewobj Cell", false),
        ("ldloc empty", false),
    ] {
        let body = format!(
            ".local System.Object empty\nldloca empty\ninitobj System.Object\nldc.i4 42\nbox Int32\n{right}\n{EQUALS}"
        );
        assert_eq!(
            run(&body, CELL, "Boolean", 8).unwrap().value,
            Value::Boolean(expected)
        );
    }
    let direct = "call instance System.Object::Equals(System.Object)";
    assert_eq!(
        run(
            &format!("ldc.i4 42\nbox Int32\nldc.i4 42\nbox Int32\n{direct}"),
            "",
            "Boolean",
            8
        )
        .unwrap()
        .value,
        Value::Boolean(false)
    );
}

#[test]
fn boxed_int32_hash_matches_value_across_boundaries_and_collection() {
    for value in [i32::MIN, -1, 0, 1, i32::MAX] {
        assert_eq!(
            run(
                &format!("ldc.i4 {value}\nbox Int32\n{HASH}"),
                "",
                "Int32",
                8
            )
            .unwrap()
            .value,
            Value::Int32(value)
        );
    }
    let mut body = ".local Int32 original\n.local System.Object boxed\nldc.i4 42\nstloc original\nldloc original\nbox Int32\nstloc boxed\nldc.i4 7\nstloc original\n".to_string();
    for _ in 0..8 {
        body.push_str("ldc.i4 9\nbox Int32\npop\n");
    }
    body.push_str(&format!("ldloc boxed\n{HASH}"));
    let result = run(&body, "", "Int32", 2).unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert!(result.heap.collections() > 1);
}

#[test]
fn boxed_intrinsic_dispatch_does_not_claim_other_object_definitions() {
    let module = neoclr::assemble(".module Application\n.entry Main\n.type class System.Object\n.method instance virtual GetHashCode() -> Int32\nldc.i4 99\nret\n.end\n.end\n.function Main() -> Int32\nldc.i4 42\nbox Int32\ncallvirt instance System.Object::GetHashCode()\nret\n.end").unwrap();
    neoclr::verify(&module).unwrap();
    assert!(neoclr::run(&module, Limits::default()).is_err());
}

const STRUCT_KEY: &str = ".type Key\n.field Number Int32\n.method instance override readonly byref GetHashCode() -> Int32\nldarg this\nldfld 0\nret\n.end\n.end";

#[test]
fn named_struct_box_dispatch_reads_copied_payload() {
    let body = format!(".local Key source\n.local System.Object boxed\nldc.i4 42\nnewobj Key\nstloc source\nldloc source\nbox Key\nstloc boxed\nldloca source\nldflda 0\nldc.i4 99\nstobj Int32\nldloc boxed\n{HASH}");
    assert_eq!(
        run(&body, STRUCT_KEY, "Int32", 8).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn named_struct_same_name_is_not_an_object_override() {
    let declarations = STRUCT_KEY.replace("override ", "");
    assert!(run(
        &format!("ldc.i4 42\nnewobj Key\nbox Key\n{HASH}"),
        &declarations,
        "Int32",
        8
    )
    .is_err());
}

#[test]
fn named_struct_override_preserves_return_contract() {
    let declarations = STRUCT_KEY
        .replace("GetHashCode() -> Int32", "GetHashCode() -> Boolean")
        .replace("ldarg this\nldfld 0", "ldc.bool true");
    assert!(run("ldc.i4 0", &declarations, "Int32", 8).is_err());
}

#[test]
fn unboxing_is_an_exact_type_copy_and_type_test_preserves_the_box() {
    let body = format!(".local Key copy\n.local System.Object boxed\nldc.i4 42\nnewobj Key\nbox Key\nstloc boxed\nldloc boxed\nisinst Key\nldloc boxed\n{IDENTITY}\nbrfalse failed\nldloc boxed\nunbox.any Key\nstloc copy\nldloca copy\nldflda 0\nldc.i4 99\nstobj Int32\nldloc boxed\n{HASH}\nret\nfailed:\nldc.i4 -1");
    assert_eq!(
        run(&body, STRUCT_KEY, "Int32", 8).unwrap().value,
        Value::Int32(42)
    );
    for (value, code) in [
        ("ldc.i4 42\nbox Int32", FaultCode::InvalidCast),
        (
            ".local System.Object empty\nldloca empty\ninitobj System.Object\nldloc empty",
            FaultCode::NullReference,
        ),
    ] {
        assert_eq!(
            run(
                &format!("{value}\nunbox.any Key\npop\nldc.i4 0"),
                STRUCT_KEY,
                "Int32",
                8
            )
            .unwrap_err()
            .code,
            code
        );
        assert_eq!(
            run(
                &format!("{value}\nisinst Key\nref.isnull"),
                STRUCT_KEY,
                "Boolean",
                8
            )
            .unwrap()
            .value,
            Value::Boolean(true)
        );
    }
}

#[test]
fn boxed_struct_override_mutates_shared_box_and_survives_collection() {
    let declarations = STRUCT_KEY.replace("override readonly", "override").replace("ldarg this\nldfld 0", "ldarg this\nldflda 0\nldarg this\nldfld 0\nldc.i4 1\nadd\nstobj Int32\nldarg this\nldfld 0");
    let mut body = format!(".local System.Object boxed\n.local System.Object alias\nldc.i4 40\nnewobj Key\nbox Key\nstloc boxed\nldloc boxed\nstloc alias\nldloc boxed\n{HASH}\npop\n");
    for _ in 0..8 {
        body.push_str("ldc.i4 0\nbox Int32\npop\n");
    }
    body.push_str(&format!("ldloc alias\n{HASH}"));
    let result = run(&body, &declarations, "Int32", 2).unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert!(result.heap.collections() > 1);
}

#[test]
fn boxed_struct_readonly_override_cannot_write_payload() {
    let declarations = STRUCT_KEY.replace(
        "ldarg this\nldfld 0",
        "ldarg this\nldflda 0\nldc.i4 9\nstobj Int32\nldc.i4 9",
    );
    let error = run(
        &format!("ldc.i4 42\nnewobj Key\nbox Key\n{HASH}"),
        &declarations,
        "Int32",
        8,
    )
    .unwrap_err();
    assert!(error.message.contains("readonly"), "{error}");
}

#[test]
fn unbox_does_not_convert_integer_widths_or_admit_reference_targets() {
    assert_eq!(
        run("ldc.i4 42\nbox Int32\nunbox.any Int64", "", "Int64", 8)
            .unwrap_err()
            .code,
        FaultCode::InvalidCast
    );
    assert!(run(
        "ldc.i4 42\nbox Int32\nunbox.any System.Object",
        "",
        "System.Object",
        8
    )
    .is_err());
}

#[test]
fn named_struct_generic_object_overrides_are_rejected_until_reachability_is_closed() {
    let declarations = STRUCT_KEY.replace(".type Key", ".type Key<T>");
    assert!(run("ldc.i4 0", &declarations, "Int32", 8).is_err());
}

#[test]
fn boxed_boolean_equality_requires_exact_type_and_preserves_identity() {
    for left in [false, true] {
        for (right, expected) in [
            (format!("ldc.bool {left}\nbox Boolean"), true),
            (format!("ldc.bool {}\nbox Boolean", !left), false),
            (format!("ldc.i4 {}\nbox Int32", i32::from(left)), false),
            ("ldloc empty".into(), false),
            ("ldc.i4 1\nnewobj Cell".into(), false),
        ] {
            let body = format!(
                ".local System.Object empty\nldloca empty\ninitobj System.Object\nldc.bool {left}\nbox Boolean\n{right}\n{EQUALS}"
            );
            assert_eq!(
                run(&body, CELL, "Boolean", 8).unwrap().value,
                Value::Boolean(expected)
            );
        }
        for call in [
            IDENTITY,
            "call instance System.Object::Equals(System.Object)",
        ] {
            let body =
                format!("ldc.bool {left}\nbox Boolean\nldc.bool {left}\nbox Boolean\n{call}");
            assert_eq!(
                run(&body, "", "Boolean", 8).unwrap().value,
                Value::Boolean(false)
            );
        }
    }
}

#[test]
fn boxed_boolean_hash_preserves_copied_value_through_collection() {
    for value in [false, true] {
        let mut body = format!(
            ".local Boolean original\n.local System.Object boxed\nldc.bool {value}\nstloc original\nldloc original\nbox Boolean\nstloc boxed\nldc.bool {}\nstloc original\n",
            !value
        );
        for _ in 0..8 {
            body.push_str("ldc.bool false\nbox Boolean\npop\n");
        }
        body.push_str(&format!("ldloc boxed\n{HASH}"));
        let result = run(&body, "", "Int32", 2).unwrap();
        assert_eq!(result.value, Value::Int32(i32::from(value)));
        assert!(result.heap.collections() > 1);
    }
}

#[test]
fn boxed_struct_traces_reference_fields_after_source_is_cleared() {
    let declarations = format!("{CELL}\n.type Holder\n.field Child Cell\n.method instance override readonly byref GetHashCode() -> Int32\nldarg this\nldfld 0\nldfld Cell::Number\nret\n.end\n.end");
    let mut body = ".local Holder source\n.local System.Object boxed\nldc.i4 42\nnewobj Cell\nnewobj Holder\nstloc source\nldloc source\nbox Holder\nstloc boxed\nldloca source\ninitobj Holder\n".to_string();
    for _ in 0..8 {
        body.push_str("ldc.i4 0\nbox Int32\npop\n");
    }
    body.push_str(&format!("ldloc boxed\n{HASH}"));
    let result = run(&body, &declarations, "Int32", 3).unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert!(result.heap.collections() > 1);
    assert!(
        result.heap.is_empty(),
        "completed scalar result must not retain the box or its child"
    );
}

#[test]
fn boxed_int64_uses_full_payload_exact_type_and_dotnet_hash() {
    let mut body = ".local System.Object empty\nldloca empty\ninitobj System.Object\n".to_owned();
    for value in [
        i64::MIN,
        -4294967296,
        -1,
        0,
        1,
        4294967296,
        4294967297,
        i64::MAX,
    ] {
        body.push_str(&format!(
            "ldc.i8 {value}\nbox Int64\nldc.i8 {value}\nbox Int64\n{EQUALS}\nbrfalse failed\n"
        ));
        // Different halves can have the same XOR hash; equality must still differ.
        let different = value ^ 0x0000_0001_0000_0001;
        body.push_str(&format!(
            "ldc.i8 {value}\nbox Int64\nldc.i8 {different}\nbox Int64\n{EQUALS}\nbrtrue failed\n"
        ));
        let expected = (value as i32) ^ ((value >> 32) as i32);
        body.push_str(&format!(
            "ldc.i8 {value}\nbox Int64\n{HASH}\nldc.i4 {expected}\nceq\nbrfalse failed\n"
        ));
    }
    for other in [
        "ldc.i4 1\nbox Int32",
        "ldc.bool true\nbox Boolean",
        "ldloc empty",
    ] {
        body.push_str(&format!(
            "ldc.i8 1\nbox Int64\n{other}\n{EQUALS}\nbrtrue failed\n"
        ));
    }
    body.push_str(&format!("ldc.i8 1\nbox Int64\nldc.i8 1\nbox Int64\n{IDENTITY}\nbrtrue failed\nldc.bool true\nret\nfailed:\nldc.bool false"));
    assert_eq!(
        run(&body, "", "Boolean", 16).unwrap().value,
        Value::Boolean(true)
    );
}

#[test]
fn boxed_int64_copy_and_hash_survive_collection() {
    let mut body = ".local Int64 original\n.local System.Object boxed\nldc.i8 4294967297\nstloc original\nldloc original\nbox Int64\nstloc boxed\nldc.i8 9\nstloc original\n".to_owned();
    for _ in 0..20 {
        body.push_str("ldc.i8 0\nbox Int64\npop\n");
    }
    body.push_str(&format!("ldloc boxed\n{HASH}\nldc.i4 0\nceq\nbrfalse failed\nldloc boxed\nunbox.any Int64\nldc.i8 4294967297\nceq\nret\nfailed:\nldc.bool false"));
    let result = run(&body, "", "Boolean", 8).unwrap();
    assert_eq!(result.value, Value::Boolean(true));
    assert!(result.heap.collections() > 0);
    assert!(result.heap.is_empty());
}

#[test]
fn floating_object_equality_and_hash_follow_exact_type_value_contracts() {
    let mut body = ".local System.Object empty\nldloca empty\ninitobj System.Object\n".to_owned();
    for (opcode, ty, nan_hash) in [
        ("ldc.r4", "Single", 0x7f800000),
        ("ldc.r8", "Double", 0x7ff00000),
    ] {
        for (left, right, equal) in [
            ("NaN", "NaN", true),
            ("0", "-0", true),
            ("inf", "inf", true),
            ("inf", "-inf", false),
            ("NaN", "0", false),
            ("1.5", "1.5", true),
            ("1.5", "1.25", false),
        ] {
            let branch = if equal { "brfalse" } else { "brtrue" };
            body.push_str(&format!(
                "{opcode} {left}\nbox {ty}\n{opcode} {right}\nbox {ty}\n{EQUALS}\n{branch} failed\n"
            ));
        }
        for (value, expected) in [("NaN", nan_hash), ("0", 0), ("-0", 0)] {
            body.push_str(&format!(
                "{opcode} {value}\nbox {ty}\n{HASH}\nldc.i4 {expected}\nceq\nbrfalse failed\n"
            ));
        }
        for other in ["ldc.i4 1\nbox Int32", "ldloc empty"] {
            body.push_str(&format!(
                "{opcode} 1\nbox {ty}\n{other}\n{EQUALS}\nbrtrue failed\n"
            ));
        }
        body.push_str(&format!("{opcode} 1\nbox {ty}\n{opcode} 1\nbox {ty}\n{IDENTITY}\nbrtrue failed\n{opcode} NaN\n{opcode} NaN\nceq\nbrtrue failed\n"));
    }
    body.push_str(&format!("ldc.r4 1\nbox Single\nldc.r8 1\nbox Double\n{EQUALS}\nbrtrue failed\nldc.bool true\nret\nfailed:\nldc.bool false"));
    let result = run(&body, "", "Boolean", 12).unwrap();
    assert_eq!(result.value, Value::Boolean(true));
    assert!(result.heap.collections() > 0);
    assert!(result.heap.is_empty());
}
