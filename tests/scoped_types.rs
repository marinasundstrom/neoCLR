use neoclr::{
    Limits, LoadedProgram, Value,
    assembler::{assemble_modules, parse_type},
    library, load_modules,
    metadata::Type,
};

const MODELS: &str = ".module Models\n.references ()\n.type Box<T>\n.field Value T\n.method static Create(T value) -> [Models]Box<T>\nldarg value\nnewobj [Models]Box<T>\nret\n.end\n.end";
const APP: &str = ".module App\n.references (Models)\n.entry Main\n.function Main() -> [System]Int32\n.local [Models]Box<[System]Int32> value\nldc.i4 42\ncall [Models]Box<[System]Int32>::Create([System]Int32) @ Models:0\nstloc value\nldloc value\nldfld [Models]Box<[System]Int32>::Value\nret\n.end";

fn prepare(sources: &[&str]) -> Result<LoadedProgram, neoclr::Fault> {
    let modules = assemble_modules(sources)?;
    LoadedProgram::with_modules(&modules[0], library::system()?, &modules[1..])
}

#[test]
fn scoped_generic_signatures_calls_and_aliases_bind_without_mutating_artifacts() {
    let modules = assemble_modules(&[APP, MODELS]).unwrap();
    let original = serde_json::to_value(&modules).unwrap();
    let json: Vec<_> = modules
        .iter()
        .map(|m| serde_json::to_string(m).unwrap())
        .collect();
    assert!(json[0].contains("Scoped"));
    let loaded = load_modules(&json.iter().map(String::as_str).collect::<Vec<_>>()).unwrap();
    let program =
        LoadedProgram::with_modules(&loaded[0], library::system().unwrap(), &loaded[1..]).unwrap();
    assert!(program.verify().is_ok());
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
    assert_eq!(
        program
            .resolve_type_identity(&parse_type("[Models]Box<[System]int32>").unwrap())
            .unwrap(),
        program
            .resolve_type_identity(&parse_type("Box<Int32>").unwrap())
            .unwrap()
    );
    assert_eq!(serde_json::to_value(&modules).unwrap(), original);
}

#[test]
fn wrong_scope_fails_even_when_the_same_unqualified_name_exists() {
    for wrong in ["[Missing]Box", "[App]Box", "[System]Box"] {
        assert!(
            prepare(&[&APP.replace("[Models]Box", wrong), MODELS]).is_err(),
            "{wrong}"
        );
    }
    assert!(prepare(&[&APP.replace("[System]Int32", "[Models]Int32"), MODELS]).is_err());
    assert!(
        prepare(&[&APP.replace("(Models)", "()"), MODELS])
            .unwrap_err()
            .message
            .contains("does not reference Models")
    );
}

#[test]
fn scoped_pointers_and_memory_operands_use_the_existing_layout_contract() {
    let source = ".module App\n.references (Models)\n.entry Main\n.function Main() -> Int32\n.local [Models]Box<Int32>* pointer\nldc.i4 1\nheap.alloc [Models]Box<Int32>\nstloc pointer\nldloc pointer\ninitobj [Models]Box<Int32>\nldloc pointer\nldc.i4 42\nnewobj [Models]Box<Int32>\nstobj [Models]Box<Int32>\nldloc pointer\nldobj [Models]Box<Int32>\nldfld [Models]Box<Int32>::Value\nldloc pointer\nheap.free\npop\nret\n.end";
    let program = prepare(&[source, MODELS]).unwrap();
    assert!(program.verify().is_ok());
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn scoped_attributes_bind_constructor_owners() {
    let markers = ".module Markers\n.references ()\n.type Marker\n.method instance .ctor() -> Void\nldvoid\nret\n.end\n.end";
    let app = ".module App\n.references (Markers)\n.type Target\n.custom instance [Markers]Marker::.ctor() @ Markers:0\n.end";
    assert!(prepare(&[app, markers]).is_ok());
    assert!(prepare(&[&app.replace("[Markers]", "[App]"), markers]).is_err());
}

#[test]
fn syntax_qualifies_definitions_and_preserves_canonical_primitive_names() {
    assert_eq!(
        parse_type("[System]int32").unwrap(),
        Type::Scoped {
            module: "System".into(),
            name: "System.Int32".into(),
            arguments: vec![]
        }
    );
    for invalid in [
        "[]Box",
        "[ModelsBox",
        "[Models]",
        "[Models]!0",
        "[Models]Option<Int32>",
        "[Models][Models]Box",
        "[Bad/Name]Box",
    ] {
        assert!(parse_type(invalid).is_err(), "{invalid}");
    }
    let program = prepare(&[APP, MODELS]).unwrap();
    for ty in [
        Type::Scoped {
            module: "System".into(),
            name: "int32".into(),
            arguments: vec![],
        },
        Type::Scoped {
            module: "Models".into(),
            name: "Box".into(),
            arguments: vec![],
        },
        Type::Scoped {
            module: "System".into(),
            name: "System.Int32".into(),
            arguments: vec![Type::Void],
        },
    ] {
        assert!(program.resolve_type_identity(&ty).is_err(), "{ty:?}");
    }
}

#[test]
fn primitive_scopes_work_in_system_and_unresolved_scopes_remain_in_source() {
    let system = neoclr::assemble(".module System\n.references ()\n.type System.Int32\n.end\n.function Get() -> [System]Int32\nldc.i4 1\nret\n.end").unwrap();
    assert!(matches!(system.functions[0].returns, Type::Scoped { .. }));
    let program = LoadedProgram::new(&system).unwrap();
    assert!(program.verify().is_ok());
    assert!(
        program
            .resolve_type_identity(&parse_type("[System]Int32").unwrap())
            .is_ok()
    );
}
