use neoclr::{
    Limits, LoadedProgram, Value, assemble,
    assembler::{assemble_modules, parse_function_ref},
    metadata::{Type, Visibility},
};
const SAMPLE: &str = include_str!("../examples/field_access.neoil");
const VAULT: &str = ".type Vault\n.field private Stored Int32\n.method static Create() -> Vault\nldc.i4 42\nnewobj Vault\nret\n.end\n.end";

#[test]
fn generic_private_fields_round_trip_and_explicit_value_updates_preserve_copies() {
    let module = assemble(SAMPLE).unwrap();
    assert_eq!(module.types[0].fields[0].visibility, Visibility::Private);
    let fields = module
        .instantiated_fields(&Type::Constructed {
            definition: "Box".into(),
            arguments: vec![Type::Int32],
        })
        .unwrap();
    assert_eq!(fields[0].visibility, Visibility::Private);
    assert_eq!(fields[0].ty, Type::Int32);
    let loaded = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    let program = LoadedProgram::new(&loaded).unwrap();
    program.verify().unwrap();
    assert_eq!(program.run(Limits::default()).unwrap().output, ["21", "42"]);
}

#[test]
fn direct_aggregate_construction_requires_access_to_every_initialized_field() {
    for body in [
        "ldc.i4 99\nnewobj Vault\npop\nldvoid\nret",
        "ldvoid\nret\nldc.i4 99\nnewobj Vault\nret",
    ] {
        let source = format!(".module App\n{VAULT}\n.function Bad() -> Void\n{body}\n.end");
        assert!(
            assemble(&source)
                .unwrap_err()
                .message
                .contains("field access denied")
        );
    }
}

#[test]
fn reads_writes_and_addresses_are_checked_by_verifier_and_unverified_execution() {
    for (returns, body) in [
        ("Int32", "call Vault::Create()\nldfld 0\nret"),
        ("Int32", "call Vault::Create()\nldfld Vault::Stored\nret"),
        ("Vault", "call Vault::Create()\nldc.i4 99\nstfld 0\nret"),
        ("Int32*", "ptr.null Vault\nldflda 0\nret"),
    ] {
        let source = format!(
            ".module App\n.entry Main\n{VAULT}\n.function Main() -> {returns}\n{body}\n.end"
        );
        let module = assemble(&source).unwrap();
        let program = LoadedProgram::new(&module).unwrap();
        assert!(
            program
                .verify()
                .unwrap_err()
                .message
                .contains("field access denied")
        );
        let fault = program.run(Limits::default()).unwrap_err();
        assert!(fault.message.contains("field access denied"), "{fault}");
        assert_eq!(fault.stack_trace.unwrap().frames[0].function.name, "Main");
    }
}

#[test]
fn internal_fields_allow_same_module_functions_but_reject_foreign_access() {
    let library = ".module Models\n.revision r1\n.type Cell\n.field internal Stored Int32\n.end\n.function Create() -> Cell\nldc.i4 42\nnewobj Cell\nret\n.end\n.function Read(Cell value) -> Int32\nldarg value\nldfld 0\nret\n.end";
    let app = ".module App\n.references (Models#r1)\n.entry Main\n.function Main() -> Int32\ncall Create()\ncall Read(Cell)\nret\n.end";
    let modules = assemble_modules(&[app, library]).unwrap();
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
    let bad = app.replace("call Read(Cell)", "ldfld 0");
    let modules = assemble_modules(&[&bad, library]).unwrap();
    let program = LoadedProgram::with_modules(
        &modules[0],
        neoclr::library::system().unwrap(),
        &modules[1..],
    )
    .unwrap();
    assert!(program.verify().is_err());
    assert!(
        program
            .run(Limits::default())
            .unwrap_err()
            .message
            .contains("field access denied")
    );
    assert!(
        assemble_modules(&[
            &app.replace("call Create()", "ldc.i4 42\nnewobj Cell"),
            library
        ])
        .is_err()
    );
}

#[test]
fn host_record_import_remains_an_explicit_trusted_data_boundary() {
    let program = LoadedProgram::new(&assemble(SAMPLE).unwrap()).unwrap();
    let read = program
        .resolve_function(&parse_function_ref("instance Box<Int32>::Read()").unwrap())
        .unwrap();
    let value = Value::Object {
        ty: Type::Constructed {
            definition: "Box".into(),
            arguments: vec![Type::Int32],
        },
        fields: vec![Value::Int32(77)],
    };
    assert_eq!(
        read.invoke_instance(value, vec![], Limits::default())
            .unwrap()
            .value,
        Value::Int32(77)
    );
}

#[test]
fn omitted_field_visibility_is_public_and_unknown_json_flags_are_rejected() {
    let module = assemble(".module App\n.type Point\n.field X Int32\n.end").unwrap();
    let json = serde_json::to_string(&module).unwrap();
    assert!(!json.contains("visibility"));
    assert_eq!(
        neoclr::load(&json).unwrap().types[0].fields[0].visibility,
        Visibility::Public
    );
    let mut json = serde_json::to_value(module).unwrap();
    json["types"][0]["fields"][0]["visibility"] = "protected".into();
    assert!(neoclr::load(&json.to_string()).is_err());
}
