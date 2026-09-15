include!("support/calendar_suite.rs");

fn calendar_library() -> String {
    let output = std::process::Command::new("python3")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(["-c", "import runpy; m=runpy.run_path('docs/experiments/raven-target/collection_library.py'); print(m['build'](m['ROOT'] / 'runtime/System.neoil'))"])
        .output().unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap()
}

fn with_library(source: &str, text: &str) -> LoadedProgram {
    let library = neoclr::assemble(text).unwrap();
    let module = neoclr::assembler::read_modules(
        &[neoclr::assembler::ModuleInput::Source(source)],
        &library,
    )
    .unwrap()
    .remove(0);
    LoadedProgram::with_library(&module, &library).unwrap()
}

fn load_calendar_program(source: &str) -> LoadedProgram {
    with_library(source, &calendar_library())
}

#[test]
fn readonly_time_body_cannot_mutate_its_receiver() {
    let library = calendar_library().replace(
        ".method instance readonly byref get_Ticks() -> Int64\n",
        ".method instance readonly byref get_Ticks() -> Int64\nldarg 0\nldc.i8 0\nstfld System.Time::StoredTicks\npop\n");
    let p = with_library(".module Probe\n.function Read(System.Time& value) -> Int64\nldarg value\ncall instance System.Time::get_Ticks()\nret\n.end", &library);
    let error = p.verify().unwrap_err().to_string();
    assert!(error.contains("readonly"), "{error}");
}

#[test]
fn time_private_constructor_is_inaccessible_outside_its_type() {
    let library = neoclr::assemble(&calendar_library()).unwrap();
    let error = neoclr::assembler::read_modules(
        &[neoclr::assembler::ModuleInput::Source(".module Probe\n.function Make() -> System.Time\nldc.i8 -1\nnewobj instance System.Time::.ctor(Int64)\nret\n.end")],
        &library).err().expect("private construction must be rejected").to_string();
    assert!(error.contains("access denied"), "{error}");
}
