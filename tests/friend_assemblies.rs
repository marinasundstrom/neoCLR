use neoclr::{
    Limits, LoadedProgram, Value,
    assembler::assemble_modules,
    library, load_modules,
    metadata::{CustomAttribute, Visibility},
};

fn fixture() -> Vec<neoclr::Module> {
    let app = ".module Friend\n.references (Library)\n.entry Main\n.function Main() -> Int32\nldc.i4 7\nnewobj Hidden\nldfld 0\ncall Hidden::Number()\nadd\nret\n.end";
    let lib = ".module Library\n.references ()\n.type System.Runtime.CompilerServices.InternalsVisibleToAttribute\n.method instance .ctor(String) -> Void\nfault \"must not execute\"\n.end\n.end\n.type Hidden\n.field Value Int32\n.method static Number() -> Int32\nldc.i4 35\nret\n.end\n.end";
    let mut modules = assemble_modules(&[app, lib]).unwrap();
    let library = &mut modules[1];
    let hidden = library
        .types
        .iter_mut()
        .find(|t| t.name == "Hidden")
        .unwrap();
    hidden.visibility = Visibility::Internal;
    hidden.fields[0].visibility = Visibility::Internal;
    library
        .functions
        .iter_mut()
        .find(|f| f.name == "Hidden.Number")
        .unwrap()
        .visibility = Visibility::Internal;
    let grant: CustomAttribute = serde_json::from_value(serde_json::json!({
        "constructor": {"name":"System.Runtime.CompilerServices.InternalsVisibleToAttribute..ctor", "owner":{"Named":"System.Runtime.CompilerServices.InternalsVisibleToAttribute"}, "instance":true, "parameters":["String"]},
        "arguments":[{"String":"Friend"}]
    })).unwrap();
    library
        .assemblies
        .iter_mut()
        .find(|a| a.name == "Library")
        .unwrap()
        .custom_attributes
        .push(grant);
    modules
}
fn prepare(modules: &[neoclr::Module]) -> Result<LoadedProgram, neoclr::Fault> {
    let json: Vec<_> = modules
        .iter()
        .map(|m| serde_json::to_string(m).unwrap())
        .collect();
    let loaded = load_modules(&json.iter().map(String::as_str).collect::<Vec<_>>())?;
    LoadedProgram::with_modules(&loaded[0], library::system()?, &loaded[1..])
}
#[test]
fn named_friend_executes_internal_type_field_and_method() {
    let program = prepare(&fixture()).unwrap();
    assert!(program.verify().is_ok());
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}
#[test]
fn missing_wrong_or_qualified_grants_do_not_authorize_access() {
    for name in [
        "Other",
        "Friend, Version=1.0.0.0",
        "Friend, PublicKeyToken=null",
        "Friend, PublicKey=00",
        "",
    ] {
        let mut modules = fixture();
        modules[1]
            .assemblies
            .iter_mut()
            .find(|a| a.name == "Library")
            .unwrap()
            .custom_attributes[0]
            .arguments[0] = neoclr::metadata::AttributeArgument::String(Some(name.into()));
        assert!(prepare(&modules).is_err(), "{name}");
    }
    let mut modules = fixture();
    modules[1]
        .assemblies
        .iter_mut()
        .find(|a| a.name == "Library")
        .unwrap()
        .custom_attributes
        .clear();
    assert!(prepare(&modules).is_err());
}
#[test]
fn friend_does_not_gain_private_access() {
    for field in [true, false] {
        let mut modules = fixture();
        if field {
            modules[1]
                .types
                .iter_mut()
                .find(|t| t.name == "Hidden")
                .unwrap()
                .fields[0]
                .visibility = Visibility::Private;
        } else {
            modules[1]
                .functions
                .iter_mut()
                .find(|f| f.name == "Hidden.Number")
                .unwrap()
                .visibility = Visibility::Private;
        }
        let result = prepare(&modules);
        assert!(result.is_err() || result.unwrap().run(Limits::default()).is_err());
    }
}

#[test]
fn grants_are_directional_and_ascii_names_ignore_case() {
    let mut modules = fixture();
    let assembly = modules[1]
        .assemblies
        .iter_mut()
        .find(|a| a.name == "Library")
        .unwrap();
    assembly.custom_attributes[0].arguments[0] =
        neoclr::metadata::AttributeArgument::String(Some("fRiEnD".into()));
    assert_eq!(
        prepare(&modules)
            .unwrap()
            .run(Limits::default())
            .unwrap()
            .value,
        Value::Int32(42)
    );
    let mut grant = assembly_grant(&mut modules);
    grant.arguments[0] = neoclr::metadata::AttributeArgument::String(Some("Library".into()));
    modules[0]
        .assemblies
        .iter_mut()
        .find(|a| a.name == "Friend")
        .unwrap()
        .custom_attributes
        .push(grant);
    assert!(prepare(&modules).is_err());
}
fn assembly_grant(modules: &mut [neoclr::Module]) -> CustomAttribute {
    modules[1]
        .assemblies
        .iter_mut()
        .find(|a| a.name == "Library")
        .unwrap()
        .custom_attributes
        .pop()
        .unwrap()
}
