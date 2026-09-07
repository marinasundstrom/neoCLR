use neoclr::{
    Limits, LoadedProgram, Value, assembler::assemble_modules, library, load_modules,
    metadata::Type,
};

const MAIN: &str = ".module App\n.references (Helpers)\n.entry Main\n.function Main() -> Int32\ncall Helpers.Get()\nret\n.end";
const HELPERS: &str =
    ".module Helpers\n.references ()\n.function Helpers.Get() -> Int32\nldc.i4 42\nret\n.end";

fn prepare(sources: &[&str]) -> Result<LoadedProgram, neoclr::Fault> {
    let modules = assemble_modules(sources)?;
    LoadedProgram::with_modules(&modules[0], library::system()?, &modules[1..])
}

#[test]
fn explicit_references_roundtrip_and_preserve_execution() {
    let modules = assemble_modules(&[MAIN, HELPERS]).unwrap();
    let json: Vec<_> = modules
        .iter()
        .map(|m| serde_json::to_string(m).unwrap())
        .collect();
    let loaded = load_modules(&json.iter().map(String::as_str).collect::<Vec<_>>()).unwrap();
    assert_eq!(loaded[0].references, Some(vec!["Helpers".into()]));
    assert_eq!(loaded[1].references, Some(vec![]));
    let program =
        LoadedProgram::with_modules(&loaded[0], library::system().unwrap(), &loaded[1..]).unwrap();
    assert!(program.verify().is_ok());
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
    assert!(program.resolve_type_identity(&Type::Int32).is_ok());
}

#[test]
fn empty_list_rejects_calls_even_with_an_explicit_definition_row() {
    for call in ["Helpers.Get()", "Helpers.Get() @ Helpers:0"] {
        let source = MAIN
            .replace("(Helpers)", "()")
            .replace("Helpers.Get()", call);
        assert!(
            prepare(&[&source, HELPERS])
                .unwrap_err()
                .message
                .contains("does not reference Helpers")
        );
    }
}

#[test]
fn references_are_direct_and_checked_in_each_declaring_module() {
    let middle = ".module Middle\n.references (Helpers)\n.function Middle.Get() -> Int32\ncall Helpers.Get()\nret\n.end";
    let root = MAIN
        .replace("(Helpers)", "(Middle)")
        .replace("Helpers.Get()", "Middle.Get()");
    assert!(prepare(&[&root, middle, HELPERS]).is_ok());
    assert!(
        prepare(&[&MAIN.replace("(Helpers)", "(Middle)"), middle, HELPERS])
            .unwrap_err()
            .message
            .contains("does not reference Helpers")
    );
    assert!(
        prepare(&[&root, &middle.replace("(Helpers)", "()"), HELPERS])
            .unwrap_err()
            .message
            .contains("module Middle does not reference Helpers")
    );
}

#[test]
fn type_fields_signatures_operands_and_queries_require_references() {
    let types = ".module Types\n.references ()\n.type Marker\n.end";
    for declaration in [
        ".type Local\n.field Value Ptr<System.Option<Marker>>\n.end",
        ".function F(Marker value) -> Void\nldvoid\nret\n.end",
        ".function F() -> Void\n.local Marker value\nldvoid\nret\n.end",
        ".function F() -> Void\nsizeof Marker\npop\nldvoid\nret\n.end",
    ] {
        let source = format!(".module App\n.references ()\n{declaration}");
        assert!(
            prepare(&[&source, types])
                .unwrap_err()
                .message
                .contains("does not reference Types")
        );
        assert!(
            prepare(&[
                &source.replace(".references ()", ".references (Types)"),
                types
            ])
            .is_ok()
        );
    }
    let program = prepare(&[".module App\n.references ()", types]).unwrap();
    assert!(
        program
            .resolve_type_identity(&Type::Named("Marker".into()))
            .unwrap_err()
            .message
            .contains("does not reference Types")
    );
}

#[test]
fn attributes_and_entry_references_do_not_bypass_module_lists() {
    let marker = ".module Markers\n.references ()\n.type Marker\n.method instance .ctor() -> Void\nldvoid\nret\n.end\n.end";
    for declaration in [
        ".type Target\n.custom instance Marker::.ctor()\n.end",
        ".function Target() -> Void\n.custom instance Marker::.ctor() @ Markers:0\nldvoid\nret\n.end",
    ] {
        assert!(
            prepare(&[
                &format!(".module App\n.references ()\n{declaration}"),
                marker
            ])
            .unwrap_err()
            .message
            .contains("does not reference Markers")
        );
    }
    assert!(
        prepare(&[".module App\n.references ()\n.entry Helpers.Get", HELPERS])
            .unwrap_err()
            .message
            .contains("does not reference Helpers")
    );
}

#[test]
fn lists_reject_missing_unused_duplicates_self_and_invalid_syntax() {
    for directive in [
        ".references (Missing)",
        ".references (Helpers, Helpers)",
        ".references (App)",
        ".references (Helpers,)",
        ".references Helpers",
        ".references ()\n.references ()",
    ] {
        assert!(
            prepare(&[&format!(".module App\n{directive}"), HELPERS]).is_err(),
            "{directive}"
        );
    }
    let mut modules = assemble_modules(&[MAIN, HELPERS]).unwrap();
    modules[0].references = Some(vec!["".into()]);
    assert!(
        LoadedProgram::with_modules(&modules[0], library::system().unwrap(), &modules[1..])
            .is_err()
    );
    assert!(neoclr::assemble(".module System\n.references (Helpers)").is_err());
}

#[test]
fn legacy_metadata_keeps_load_set_visibility() {
    let legacy = MAIN.replace(".references (Helpers)\n", "");
    let modules = assemble_modules(&[&legacy, HELPERS]).unwrap();
    assert!(modules[0].references.is_none());
    assert!(
        !serde_json::to_value(&modules[0])
            .unwrap()
            .as_object()
            .unwrap()
            .contains_key("references")
    );
    assert_eq!(
        prepare(&[&legacy, HELPERS])
            .unwrap()
            .run(Limits::default())
            .unwrap()
            .value,
        Value::Int32(42)
    );
}
