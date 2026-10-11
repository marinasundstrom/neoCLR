//! A provider and its consumer may each declare the same existing runtime service.
use std::{fs, process::Command};

#[test]
fn separately_declared_runtime_async_services_keep_distinct_native_identities() {
    let directory = std::env::temp_dir().join(format!("neoclr-sql-services-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let app = directory.join("app.neox");
    let library = directory.join("provider.neox");
    let output = directory.join("app.o");
    let system = directory.join("system.neox");
    let seed = neoclr::assemble(".module System\n.references ()\n").unwrap();
    fs::write(&system, neoclr::metadata_container::write_module(&seed).unwrap()).unwrap();
    let declaration = ".function neoCLR.Runtime.RuntimeAsyncCreate() -> Int64\n.methodimpl InternalCall\n.end\n";
    let provider_source = format!(".module SqlProvider\n.references ()\n{declaration}\n.function ProviderValue() -> Int64\ncall neoCLR.Runtime.RuntimeAsyncCreate()\nret\n.end\n");
    let consumer_source = format!(".module SqlConsumer\n.references (SqlProvider)\n{declaration}\n.function Main() -> Int32\ncall ProviderValue()\npop\ncall neoCLR.Runtime.RuntimeAsyncCreate()\npop\nldc.i4 42\nret\n.end\n");
    let modules = neoclr::assembler::read_modules(&[
        neoclr::assembler::ModuleInput::Source(&provider_source),
        neoclr::assembler::ModuleInput::Source(&consumer_source),
    ], &seed).unwrap();
    fs::write(&library, neoclr::metadata_container::write_module(&modules[0]).unwrap()).unwrap();
    fs::write(&app, neoclr::metadata_container::write_module(&modules[1]).unwrap()).unwrap();
    let run = |admit| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc"));
        command.args(["--closed-world"]).arg(&app).arg("Main").arg(&output)
            .arg("--system").arg(&system)
            .arg("--module").arg(&library)
            .args(["--compile-system", "--reference-arena", "--native-gc", "--native-stack-budget"]);
        if admit { command.arg("--bind-runtime-async"); }
        command.output().unwrap()
    };
    let rejected = run(false);
    assert!(!rejected.status.success(), "Unadmitted service must remain rejected");
    assert!(!output.exists());
    let accepted = run(true);
    assert!(accepted.status.success(), "{}", String::from_utf8_lossy(&accepted.stderr));
    assert!(output.exists());
    fs::remove_dir_all(directory).unwrap();
}
