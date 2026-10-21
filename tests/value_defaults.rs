use neoclr::{LoadedProgram, Value};
const EQUALS: &str = "callvirt instance System.Object::Equals(System.Object)";
const HASH: &str = "callvirt instance System.Object::GetHashCode()";
const SEED: &str = r#"
.module System
.type class abstract System.Object
.method instance virtual Equals(System.Object other) -> Boolean
ldarg 0
ldarg 1
call neoCLR.Runtime.ObjectEquals(System.Object,System.Object)
ret
.end
.method instance virtual GetHashCode() -> Int32
ldarg 0
call neoCLR.Runtime.ObjectIdentityHash(System.Object)
ret
.end
.end
.function neoCLR.Runtime.ObjectEquals(System.Object,System.Object) -> Boolean
.methodimpl InternalCall
.end
.function neoCLR.Runtime.ObjectIdentityHash(System.Object) -> Int32
.methodimpl InternalCall
.end
"#;
fn run(
    body: &str,
    declarations: &str,
    returns: &str,
    heap_objects: usize,
) -> Result<neoclr::Execution, neoclr::Fault> {
    let library = neoclr::assemble(SEED)?;
    let source = format!(
        ".module Defaults\n.entry Main\n{declarations}\n.function Main() -> {returns}\n{body}\nret\n.end"
    );
    let app = neoclr::assembler::read_modules(
        &[neoclr::assembler::ModuleInput::Source(&source)],
        &library,
    )?
    .remove(0);
    let program = LoadedProgram::with_library(&app, &library)?;
    program.verify()?;
    program.run(neoclr::Limits {
        heap_objects,
        ..Default::default()
    })
}

#[test]
fn default_values_compare_nested_fields_and_preserve_field_overrides() {
    let declarations = ".type Inner\n.field Number Int64\n.end\n.type Outer\n.field Nested Inner\n.field Ref System.Object\n.end\n.type class Key\n.extends System.Object\n.method instance override Equals(System.Object other) -> Boolean\nldc.bool true\nret\n.end\n.method instance override GetHashCode() -> Int32\nldc.i4 42\nret\n.end\n.end";
    let value = "ldc.i8 4294967297\nnewobj Inner\nnewobj Key\ncastclass System.Object\nnewobj Outer\nbox Outer";
    assert_eq!(
        run(
            &format!("{value}\n{value}\n{EQUALS}"),
            declarations,
            "Boolean",
            32
        )
        .unwrap()
        .value,
        Value::Boolean(true)
    );
    assert_eq!(
        run(
            &format!("{value}\n{HASH}\n{value}\n{HASH}\nceq"),
            declarations,
            "Boolean",
            32
        )
        .unwrap()
        .value,
        Value::Boolean(true)
    );
}

#[test]
fn empty_defaults_require_exact_type_and_null_fields_are_values() {
    let declarations =
        ".type Empty\n.end\n.type Other\n.end\n.type NullableField\n.field Ref System.Object\n.end";
    for (right, expected) in [
        ("newobj Empty\nbox Empty", true),
        ("newobj Other\nbox Other", false),
    ] {
        assert_eq!(
            run(
                &format!("newobj Empty\nbox Empty\n{right}\n{EQUALS}"),
                declarations,
                "Boolean",
                16
            )
            .unwrap()
            .value,
            Value::Boolean(expected)
        );
    }
    let prefix = ".local System.Object empty\nldloca empty\ninitobj System.Object\n";
    let value = "ldloc empty\nnewobj NullableField\nbox NullableField";
    assert_eq!(
        run(
            &format!("{prefix}{value}\n{value}\n{EQUALS}"),
            declarations,
            "Boolean",
            16
        )
        .unwrap()
        .value,
        Value::Boolean(true)
    );
    assert_eq!(
        run(
            &format!("{prefix}{value}\nldloc empty\n{EQUALS}"),
            declarations,
            "Boolean",
            16
        )
        .unwrap()
        .value,
        Value::Boolean(false)
    );
}

#[test]
fn defaults_preserve_float_field_contracts() {
    for (ty, value, different) in [
        ("Single", "ldc.r4 NaN", "ldc.r4 0"),
        ("Double", "ldc.r8 -0", "ldc.r8 1"),
    ] {
        let declarations = format!(".type Floating\n.field Number {ty}\n.end");
        let boxed = format!("{value}\nnewobj Floating\nbox Floating");
        assert_eq!(
            run(
                &format!("{boxed}\n{boxed}\n{EQUALS}"),
                &declarations,
                "Boolean",
                32
            )
            .unwrap()
            .value,
            Value::Boolean(true)
        );
        assert_eq!(
            run(
                &format!("{boxed}\n{different}\nnewobj Floating\nbox Floating\n{EQUALS}"),
                &declarations,
                "Boolean",
                32
            )
            .unwrap()
            .value,
            Value::Boolean(false)
        );
        assert_eq!(
            run(
                &format!("{boxed}\n{HASH}\n{boxed}\n{HASH}\nceq"),
                &declarations,
                "Boolean",
                32
            )
            .unwrap()
            .value,
            Value::Boolean(true)
        );
    }
}

#[test]
fn defaults_use_all_integral_bits_without_field_boxes() {
    for (ty, literal) in [
        ("Byte", "ldc.i4 255\nconv.u1"),
        ("UInt32", "ldc.i4 -1\nconv.u4"),
        ("UInt64", "ldc.i8 -1\nconv.u8"),
        ("Boolean", "ldc.bool true"),
    ] {
        let declarations = format!(".type Integral\n.field Number {ty}\n.end");
        let boxed = format!("{literal}\nnewobj Integral\nbox Integral");
        let execution = run(
            &format!("{boxed}\n{boxed}\n{EQUALS}"),
            &declarations,
            "Boolean",
            2,
        )
        .unwrap();
        assert_eq!(execution.value, Value::Boolean(true));
        assert_eq!(
            run(
                &format!("{boxed}\n{HASH}\n{boxed}\n{HASH}\nceq"),
                &declarations,
                "Boolean",
                2
            )
            .unwrap()
            .value,
            Value::Boolean(true)
        );
    }
}

#[test]
fn execution_helpers_are_unique_and_do_not_add_typed_contracts() {
    let mut module =
        neoclr::assemble(&format!("{SEED}\n.type Plain\n.field Number Int32\n.end")).unwrap();
    let mut collision = module
        .functions
        .iter()
        .find(|f| f.name.ends_with(".Equals"))
        .unwrap()
        .clone();
    collision.name = "Plain.$ValueDefault0_Equals".into();
    collision.owner = None;
    collision.is_override = false;
    module.functions.push(collision);
    let source = module.clone();
    assert_eq!(neoclr::value_defaults::materialize(&mut module).unwrap(), 2);
    let helpers = &module.functions[source.functions.len()..];
    assert_eq!(helpers[0].name, "Plain.$ValueDefault1_Equals");
    assert_eq!(helpers[1].name, "Plain.$ValueDefault0_GetHashCode");
    assert_eq!(
        serde_json::to_value(&module.types).unwrap(),
        serde_json::to_value(&source.types).unwrap()
    );
    assert_eq!(neoclr::value_defaults::materialize(&mut module).unwrap(), 0);
    let roundtrip: neoclr::Module =
        serde_json::from_value(serde_json::to_value(&module).unwrap()).unwrap();
    for method in &roundtrip.functions[source.functions.len()..] {
        assert!(
            !source
                .functions
                .iter()
                .any(|contract| neoclr::value_defaults::implements_contract(method, contract))
        );
    }
}

#[test]
fn defaults_can_read_private_fields_without_exposing_them_to_callers() {
    let declarations = ".type Vault\n.field private Stored Int32\n.method static Create() -> Vault\nldc.i4 42\nnewobj Vault\nret\n.end\n.end";
    let value = "call Vault.Create()\nbox Vault";
    assert_eq!(
        run(
            &format!("{value}\n{value}\n{EQUALS}"),
            declarations,
            "Boolean",
            8
        )
        .unwrap()
        .value,
        Value::Boolean(true)
    );
    assert_eq!(
        run(&format!("{value}\n{HASH}"), declarations, "Int32", 8)
            .unwrap()
            .value,
        Value::Int32((17 * 31) ^ 42)
    );
    assert!(
        run(
            "call Vault.Create()\nldfld Vault::Stored",
            declarations,
            "Int32",
            8
        )
        .is_err()
    );
}

#[test]
fn pointer_fields_use_address_equality_and_hash_without_dereferencing_or_boxing() {
    let declarations = ".type PointerValue\n.field Address Int32*\n.end";
    for pointer in ["ptr.null Int32", "ldc.i4 1\nheap.alloc Int32"] {
        let setup = format!(".local Int32* address\n{pointer}\nstloc address\n");
        let value = "ldloc address\nnewobj PointerValue\nbox PointerValue";
        for body in [
            format!("{setup}{value}\n{value}\n{EQUALS}"),
            format!("{setup}{value}\n{HASH}\n{value}\n{HASH}\nceq"),
        ] {
            assert_eq!(
                run(&body, declarations, "Boolean", 2).unwrap().value,
                Value::Boolean(true)
            );
        }
    }
    assert_eq!(run(
        &format!("ldc.i4 1\nheap.alloc Int32\nnewobj PointerValue\nbox PointerValue\nptr.null Int32\nnewobj PointerValue\nbox PointerValue\n{EQUALS}"),
        declarations, "Boolean", 2).unwrap().value, Value::Boolean(false));
}
