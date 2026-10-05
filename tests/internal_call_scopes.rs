use neoclr::{
    Limits, LoadedProgram, Value, assemble,
    assembler::{ModuleInput, read_modules},
};

const SERVICE: &str = ".function internal neoCLR.Runtime.StringCompareOrdinal(String left,String right) -> Int32\n.methodimpl InternalCall\n.end\n";

fn system() -> neoclr::Module {
    assemble(&format!(".module System\n.references ()\n{SERVICE}.function RootCheck() -> Int32\nldstr \"same\"\nldstr \"same\"\ncall neoCLR.Runtime.StringCompareOrdinal(String,String)\nret\n.end\n")).unwrap()
}

fn application(call: &str) -> String {
    format!(
        ".module App\n.references ()\n.entry Main\n{SERVICE}.function Main() -> Int32\nldstr \"same\"\nldstr \"same\"\n{call}\ncall RootCheck() @ System:1\nadd\nret\n.end\n"
    )
}

#[test]
fn local_internal_services_keep_their_assembly_identity() {
    let system = system();
    let source = application("call neoCLR.Runtime.StringCompareOrdinal(String,String)");
    let modules = read_modules(&[ModuleInput::Source(&source)], &system).unwrap();
    let before = serde_json::to_value(&modules[0]).unwrap();
    let program = LoadedProgram::with_library(&modules[0], &system).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(0)
    );
    assert_eq!(serde_json::to_value(&modules[0]).unwrap(), before);
}

#[test]
fn explicit_external_internal_service_does_not_redirect_to_local() {
    let system = system();
    let source = application("call neoCLR.Runtime.StringCompareOrdinal(String,String) @ System:0");
    let result = read_modules(&[ModuleInput::Source(&source)], &system)
        .and_then(|modules| LoadedProgram::with_library(&modules[0], &system))
        .and_then(|program| program.verify());
    assert!(
        result.is_err(),
        "cross-assembly internal access was admitted"
    );
}

#[test]
fn duplicate_local_or_incompatible_service_declarations_still_reject() {
    let system = system();
    let source = application("call neoCLR.Runtime.StringCompareOrdinal(String,String)");
    for invalid in [
        format!("{source}{SERVICE}"),
        source.replace("-> Int32\n.methodimpl", "-> Boolean\n.methodimpl"),
    ] {
        assert!(read_modules(&[ModuleInput::Source(&invalid)], &system).is_err());
    }
}

#[test]
fn symbolic_call_without_a_local_service_stays_ambiguous() {
    let system = system();
    let source = application("call neoCLR.Runtime.StringCompareOrdinal(String,String)")
        .replacen(SERVICE, "", 1);
    let other = format!(".module Services\n{SERVICE}");
    let error = read_modules(
        &[ModuleInput::Source(&source), ModuleInput::Source(&other)],
        &system,
    )
    .unwrap_err();
    assert!(error.message.contains("ambiguous"), "{}", error.message);
}
