use neoclr::{Limits, LoadedProgram, Module, Value};
use std::{process::Command, sync::OnceLock};
fn library() -> &'static Module {
    static LIBRARY: OnceLock<Module> = OnceLock::new();
    LIBRARY.get_or_init(|| {
        let output = Command::new("python3").current_dir(env!("CARGO_MANIFEST_DIR"))
            .args(["-c", "import runpy; m=runpy.run_path('docs/experiments/raven-target/collection_library.py'); print(m['build'](m['ROOT'] / 'runtime/System.neoil'))"])
            .output().unwrap();
        assert!(output.status.success());
        neoclr::assemble(std::str::from_utf8(&output.stdout).unwrap()).unwrap()
    })
}

// The selected Raven profile has interface contracts; the bundled legacy profile
// still has value descriptors, so validate callers against the selected library.
fn assemble(source: &str) -> Result<Module, neoclr::Fault> {
    neoclr::assembler::read_modules(&[neoclr::assembler::ModuleInput::Source(source)], library())
        .map(|mut modules| modules.remove(0))
}

fn program(body: &str, result: &str) -> LoadedProgram {
    let app = assemble(&format!(
        ".module Utf8Tests\n.entry Main\n.function Main() -> {result}\n{body}\nret\n.end"
    ))
    .unwrap();
    LoadedProgram::with_library(&app, library()).unwrap()
}

#[test]
fn encoding_preserves_exact_bytes_and_obeys_array_limit() {
    let p = program(
        "ldstr \"Aé🌍\"\ncall neoCLR.Runtime.Utf8Encode(String)",
        "Byte[]",
    );
    let value = p.run(Limits::default()).unwrap().value;
    assert_eq!(
        value,
        Value::Array {
            element: neoclr::assembler::parse_type("Byte").unwrap(),
            elements: "Aé🌍".bytes().map(Value::Byte).collect(),
        }
    );
    assert!(
        p.run(Limits {
            array_elements: 6,
            ..Limits::default()
        })
        .is_err()
    );
}

#[test]
fn native_decoder_strictly_validates_unicode_and_preserves_text() {
    for bytes in [
        vec![],
        b"ASCII\0".to_vec(),
        "é🌍e\u{301}\u{feff}".as_bytes().to_vec(),
        vec![0x80],
        vec![0xc0, 0x80],
        vec![0xed, 0xa0, 0x80],
        vec![0xf4, 0x90, 0x80, 0x80],
        vec![0xf0, 0x9f, 0x8c],
        vec![0xe2, 0x28, 0xa1],
    ] {
        let mut body = format!(
            ".local arrayref<Byte> data\nldc.i4 {}\nnewarr Byte\nstloc data\n",
            bytes.len()
        );
        for (index, byte) in bytes.iter().enumerate() {
            body += &format!("ldloc data\nldc.i4 {index}\nldc.i4 {byte}\nconv.u1\nstelem Byte\n");
        }
        body += "ldloc data\ncall neoCLR.Runtime.Utf8Decode(arrayref<Byte>)";
        let actual = program(&body, "Value")
            .run(Limits::default())
            .unwrap()
            .value;
        let payload = match String::from_utf8(bytes) {
            Ok(text) => Value::String(text.into()),
            Err(_) => Value::Byte(1),
        };
        assert_eq!(actual, Value::Erased(Box::new(payload)));
    }
}

#[test]
fn utf8_services_declare_text_and_array_requirements() {
    let p = program(
        "ldstr \"text\"\ncall neoCLR.Runtime.Utf8Encode(String)",
        "Byte[]",
    );
    let graph = p
        .analyze_reachability(
            &[neoclr::assembler::parse_function_ref("Main()").unwrap()],
            4,
        )
        .unwrap();
    let services = graph.required_services();
    assert!(services.contains(&neoclr::RuntimeService::StringOperations));
    assert!(services.contains(&neoclr::RuntimeService::ManagedArrays));
}
