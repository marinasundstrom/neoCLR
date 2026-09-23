use neoclr::{Limits, LoadedProgram, Module, Value};
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
        ".module Hashing\n.entry Main\n{declarations}\n.function Main() -> {returns}\n{body}\nret\n.end"
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

#[test]
fn accumulator_matches_combine_for_signed_boundaries() {
    for (first, second) in [(0, 0), (42, 7), (i32::MIN, i32::MAX), (-1, -1)] {
        let body = format!(".local System.HashCode hash\nldloca hash\ninitobj System.HashCode\nldloca hash\nldc.i4 {first}\ncall instance System.HashCode::Add(Int32)\npop\nldloca hash\nldc.i4 {second}\ncall instance System.HashCode::Add(Int32)\npop\nldloca hash\ncall instance System.HashCode::ToHashCode()\nldc.i4 {first}\nldc.i4 {second}\ncall System.HashCode::Combine(Int32,Int32)\nceq");
        assert_eq!(
            run(&body, "", "Boolean", 32).unwrap().value,
            Value::Boolean(true)
        );
    }
}

#[test]
fn accumulator_copies_preserve_independent_state() {
    let body = ".local System.HashCode original\n.local System.HashCode copy\nldloca original\ninitobj System.HashCode\nldloca original\nldc.i4 42\ncall instance System.HashCode::Add(Int32)\npop\nldloc original\nstloc copy\nldloca copy\nldc.i4 7\ncall instance System.HashCode::Add(Int32)\npop\nldloca original\ncall instance System.HashCode::ToHashCode()\nldloca copy\ncall instance System.HashCode::ToHashCode()\nceq";
    assert_eq!(
        run(body, "", "Boolean", 32).unwrap().value,
        Value::Boolean(false)
    );
}

#[test]
fn equal_string_contents_produce_equal_hashes() {
    let body = ".local System.HashCode left\n.local System.HashCode right\nldloca left\ninitobj System.HashCode\nldloca right\ninitobj System.HashCode\nldloca left\nldstr \"Räven 🐦\"\ncall instance System.HashCode::Add(String)\npop\nldloca right\nldstr \"Räven 🐦\"\ncall instance System.HashCode::Add(String)\npop\nldloca left\ncall instance System.HashCode::ToHashCode()\nldloca right\ncall instance System.HashCode::ToHashCode()\nceq";
    assert_eq!(
        run(body, "", "Boolean", 32).unwrap().value,
        Value::Boolean(true)
    );
}
