use neoclr::{
    Limits, LoadedProgram, Module, Value,
    assembler::{ModuleInput, read_modules},
};

const ROOT: &str = r#"
.module System
.references ()
.type class abstract System.Object
.method instance virtual ToString() -> String
ldstr "root"
ret
.end
.method instance virtual Equals(System.Object other) -> Boolean
ldc.bool false
ret
.end
.method instance virtual GetHashCode() -> Int32
ldc.i4 -1
ret
.end
.end
"#;

fn modules(body: &str, declarations: &str) -> (Module, Vec<Module>) {
    let seed = neoclr::assemble(ROOT).unwrap();
    let app = format!(
        ".module App\n.references (System)\n.entry Main\n{declarations}\n.function Main() -> String\n{body}\nret\n.end"
    );
    let modules = read_modules(&[ModuleInput::Source(&app)], &seed).unwrap();
    (seed, modules)
}

#[test]
fn boxed_display_uses_the_loaded_root_definition() {
    let (seed, modules) = modules(
        "ldc.i4 42\nbox Int32\ncallvirt instance [System]System.Object::ToString()",
        "",
    );
    let before = serde_json::to_value(&modules).unwrap();
    let program = LoadedProgram::with_modules(&modules[0], &seed, &modules[1..]).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::String("42".into())
    );
    assert_eq!(serde_json::to_value(&modules).unwrap(), before);
}

#[test]
fn rootless_override_uses_the_loaded_root_slot() {
    let (seed, modules) = modules(
        "newobj Named\ncallvirt instance [System]System.Object::ToString()",
        ".type class Named\n.method instance override ToString() -> String\nldstr \"named\"\nret\n.end\n.end",
    );
    let program = LoadedProgram::with_modules(&modules[0], &seed, &modules[1..]).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::String("named".into())
    );
}

#[test]
fn boxed_equality_and_hashing_use_the_loaded_root_definition() {
    for body in [
        "ldc.i4 42\nbox Int32\nldc.i4 42\nbox Int32\ncallvirt instance [System]System.Object::Equals(System.Object)\nbrfalse failed\nldstr \"passed\"\nret\nfailed:\nldstr \"failed\"",
        "ldc.i4 42\nbox Int32\ncallvirt instance [System]System.Object::GetHashCode()\nldc.i4 42\nceq\nbrfalse failed\nldstr \"passed\"\nret\nfailed:\nldstr \"failed\"",
    ] {
        let (seed, modules) = modules(body, "");
        let program = LoadedProgram::with_modules(&modules[0], &seed, &modules[1..]).unwrap();
        program.verify().unwrap();
        assert_eq!(
            program.run(Limits::default()).unwrap().value,
            Value::String("passed".into())
        );
    }
}

#[test]
fn root_slot_identity_does_not_relax_duplicate_or_forged_ownership() {
    let (seed, modules) = modules("ldstr \"unused\"", "");
    let duplicate = neoclr::assemble(&ROOT.replace(".module System", ".module Core")).unwrap();
    assert!(LoadedProgram::with_modules(&modules[0], &seed, &[duplicate]).is_err());
    let mut forged = seed.clone();
    forged.functions[0].definition.as_mut().unwrap().module = "Core".into();
    assert!(LoadedProgram::with_modules(&modules[0], &forged, &[]).is_err());
}

#[test]
fn root_slots_cannot_be_called_through_the_wrong_scope() {
    let seed = neoclr::assemble(ROOT).unwrap();
    let app = ".module App\n.references (System)\n.entry Main\n.function Main() -> String\nldc.i4 42\nbox Int32\ncallvirt instance [Core]System.Object::ToString()\nret\n.end";
    assert!(read_modules(&[ModuleInput::Source(app)], &seed).is_err());
}

// Until the host/writer/runtime have an explicit root-ownership contract, a
// declaration's name is insufficient to opt it into intrinsic Object behavior.
#[test]
fn external_object_spelling_does_not_select_a_new_runtime_root() {
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    let core = ROOT.replace(".module System", ".module Core");
    let app = ".module App\n.references (Core)\n.entry Main\n.function Main() -> String\nldc.i4 42\nbox Int32\ncallvirt instance [Core]System.Object::ToString()\nret\n.end";
    let loaded = read_modules(
        &[ModuleInput::Source(app), ModuleInput::Source(&core)],
        &seed,
    )
    .unwrap();
    let program = LoadedProgram::with_modules(&loaded[0], &seed, &loaded[1..]).unwrap();
    program.verify().unwrap();
    assert!(program.run(Limits::default()).is_err());
}

#[test]
fn binary_load_set_preserves_selected_seed_root_dispatch() {
    let (seed, modules) = modules(
        "ldc.i4 42\nbox Int32\ncallvirt instance [System]System.Object::ToString()",
        "",
    );
    let decoded: Vec<_> = modules
        .iter()
        .map(|module| {
            let image = neoclr::metadata_container::write_module(module).unwrap();
            neoclr::metadata_container::decode_envelope(&image).unwrap()
        })
        .collect();
    let seed_image = neoclr::metadata_container::write_module(&seed).unwrap();
    let decoded_seed = neoclr::metadata_container::decode_envelope(&seed_image).unwrap();
    let program = LoadedProgram::with_modules(&decoded[0], &decoded_seed, &decoded[1..]).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::String("42".into())
    );
}
