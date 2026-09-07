use neoclr::{
    Limits, LoadedProgram, Value, assemble,
    assembler::{assemble_modules, parse_function_ref},
    metadata::Visibility,
};

const SAMPLE: &str = include_str!("../examples/accessibility.neoil");

#[test]
fn sample_round_trips_checks_access_and_runs_an_internal_entry() {
    let module = assemble(SAMPLE).unwrap();
    assert_eq!(module.functions[1].visibility, Visibility::Private);
    let json = serde_json::to_string(&module).unwrap();
    assert!(json.contains("\"visibility\":\"private\""));
    assert!(json.contains("\"visibility\":\"internal\""));
    let program = LoadedProgram::new(&neoclr::load(&json).unwrap()).unwrap();
    program.verify().unwrap();
    assert_eq!(program.run(Limits::default()).unwrap().output, ["42"]);
}

#[test]
fn private_calls_are_rejected_outside_declaring_type_even_when_unreachable() {
    for body in [
        "ldc.i4 21\ncall Gauge::Clamp(Int32)\npop\nldvoid\nret",
        "ldvoid\nret\nldc.i4 21\ncall Gauge::Clamp(Int32)\nret",
    ] {
        let source = format!("{SAMPLE}\n.function Bad() -> Void\n{body}\n.end");
        let fault = assemble(&source).unwrap_err();
        assert!(fault.message.contains("method access denied"), "{fault}");
    }
    let source = format!(
        "{SAMPLE}\n.type Other\n.method static Bad() -> Int32\nldc.i4 21\ncall Gauge::Clamp(Int32)\nret\n.end\n.end"
    );
    assert!(
        assemble(&source)
            .unwrap_err()
            .message
            .contains("method access denied")
    );
}

#[test]
fn host_invocation_requires_public_methods_and_tokens_do_not_bypass_access() {
    let module = assemble(SAMPLE).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    for signature in ["Gauge::Clamp(Int32)", "Twice(Int32)", "Main()"] {
        let target = parse_function_ref(signature).unwrap();
        assert!(
            program
                .resolve_function(&target)
                .unwrap_err()
                .message
                .contains("method access denied")
        );
        let mut bound = target;
        bound.definition = module
            .functions
            .iter()
            .find(|f| f.name == bound.name)
            .unwrap()
            .definition
            .clone();
        assert!(
            program
                .resolve_function(&bound)
                .unwrap_err()
                .message
                .contains("method access denied")
        );
    }
    let create = program
        .resolve_function(&parse_function_ref("Gauge::Create(Int32)").unwrap())
        .unwrap();
    let read = program
        .resolve_function(&parse_function_ref("instance Gauge::Read()").unwrap())
        .unwrap();
    for (input, expected) in [(-1, 0), (21, 21), (101, 100)] {
        let value = create
            .invoke(vec![Value::Int32(input)], Limits::default())
            .unwrap()
            .value;
        assert_eq!(
            read.invoke_instance(value, vec![], Limits::default())
                .unwrap()
                .value,
            Value::Int32(expected)
        );
    }
    // An analysis root is not an invocation capability.
    let graph = program
        .analyze_reachability(&[parse_function_ref("Gauge::Clamp(Int32)").unwrap()], 1)
        .unwrap();
    assert_eq!(graph.functions.len(), 1);
}

#[test]
fn internal_is_a_module_boundary_and_foreign_entry_selection_cannot_bypass_it() {
    let library = ".module Helpers\n.revision r1\n.function internal Hidden() -> Int32\nldc.i4 42\nret\n.end\n.function public Visible() -> Int32\ncall Hidden()\nret\n.end";
    let app = ".module App\n.references (Helpers#r1)\n.entry Main\n.function Main() -> Int32\ncall Visible()\nret\n.end";
    let modules = assemble_modules(&[app, library]).unwrap();
    let program = LoadedProgram::with_modules(
        &modules[0],
        neoclr::library::system().unwrap(),
        &modules[1..],
    )
    .unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
    for bad in [
        app.replace("call Visible()", "call Hidden()"),
        app.replace(".entry Main", ".entry Hidden"),
    ] {
        assert!(
            assemble_modules(&[&bad, library])
                .unwrap_err()
                .message
                .contains("method access denied")
        );
    }
    let mut bound = parse_function_ref("Hidden()").unwrap();
    bound.definition = modules[1].functions[0].definition.clone();
    assert!(
        program
            .resolve_function(&bound)
            .unwrap_err()
            .message
            .contains("method access denied")
    );
}

#[test]
fn generic_declaring_type_identity_and_private_entry_are_supported() {
    let source = ".module App\n.entry Start.Main\n.type Box<T>\n.method public static Get() -> Int32\ncall Box<T>::Secret()\nret\n.end\n.method private static Secret() -> Int32\nldc.i4 42\nret\n.end\n.end\n.type Start\n.method private static Main() -> Int32\ncall Box<Int32>::Get()\nret\n.end\n.end";
    let program = LoadedProgram::new(&assemble(source).unwrap()).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
    assert!(
        program
            .resolve_function(&parse_function_ref("Box<Int32>::Secret()").unwrap())
            .is_err()
    );
    assert!(
        program
            .resolve_function(&parse_function_ref("Start::Main()").unwrap())
            .is_err()
    );
}

#[test]
fn property_association_does_not_grant_access_to_a_private_setter() {
    let source = ".module App\n.type Settings\n.property static Value() -> Int32\n.get Settings::Read()\n.set Settings::Write(Int32)\n.end\n.method public static Read() -> Int32\nldc.i4 42\nret\n.end\n.method private static Write(Int32 value) -> Void\nldvoid\nret\n.end\n.end";
    let program = LoadedProgram::new(&assemble(source).unwrap()).unwrap();
    program.verify().unwrap();
    assert!(
        program
            .resolve_function(&parse_function_ref("Settings::Read()").unwrap())
            .is_ok()
    );
    assert!(
        program
            .resolve_function(&parse_function_ref("Settings::Write(Int32)").unwrap())
            .unwrap_err()
            .message
            .contains("method access denied")
    );
}

#[test]
fn legacy_defaults_are_public_and_invalid_visibility_is_rejected() {
    let module = assemble(".module App\n.function F() -> Void\nldvoid\nret\n.end").unwrap();
    let json = serde_json::to_string(&module).unwrap();
    assert!(!json.contains("visibility"));
    assert_eq!(
        neoclr::load(&json).unwrap().functions[0].visibility,
        Visibility::Public
    );
    for visibility in ["protected", "private", "unknown"] {
        let mut value = serde_json::to_value(&module).unwrap();
        value["functions"][0]["visibility"] = visibility.into();
        assert!(neoclr::load(&value.to_string()).is_err());
    }
    assert!(assemble(".module App\n.function private F() -> Void\nldvoid\nret\n.end").is_err());
    assert!(
        assemble(
            ".module App\n.type C\n.method static private F() -> Void\nldvoid\nret\n.end\n.end"
        )
        .is_err()
    );
}
