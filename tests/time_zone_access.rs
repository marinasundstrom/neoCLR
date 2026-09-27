#[test]
fn zone_and_offset_storage_cannot_be_forged_by_guest_il() {
    let output = std::process::Command::new("python3")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(["-c", "import runpy; m=runpy.run_path('docs/experiments/raven-target/collection_library.py'); print(m['build'](m['ROOT'] / 'runtime/System.neoil'))"])
        .output().unwrap();
    assert!(output.status.success());
    let library = neoclr::assemble(&String::from_utf8(output.stdout).unwrap()).unwrap();
    for body in [
        "ldstr \"Invented/Zone\"\nnewobj instance System.TimeZone::.ctor(String)\npop",
        ".local System.TimeOffset value\nldloca value\ninitobj System.TimeOffset\nldloc value\nldc.i4 999999\nstfld System.TimeOffset::StoredSeconds\npop",
    ] {
        let source =
            format!(".module ForgedTime\n.function Main() -> noresult\n{body}\nret\n.end\n");
        let error = neoclr::assembler::read_modules(
            &[neoclr::assembler::ModuleInput::Source(&source)],
            &library,
        )
        .and_then(|modules| {
            let program = neoclr::LoadedProgram::with_library(&modules[0], &library)?;
            program.verify()
        })
        .unwrap_err();
        assert!(error.to_string().contains("access denied"), "{error}");
    }
}
