//! Worker execution boundaries are enforced below the Raven frontend.
use neoclr::{Limits, LoadedProgram, Value, assemble};

fn library() -> &'static neoclr::Module {
    static LIBRARY: std::sync::OnceLock<neoclr::Module> = std::sync::OnceLock::new();
    LIBRARY.get_or_init(|| {
        let output = std::process::Command::new("python3")
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .args(["-c", "import runpy; m=runpy.run_path('docs/experiments/raven-target/collection_library.py'); print(m['build'](m['ROOT'] / 'runtime/System.neoil'))"])
            .output().unwrap();
        assert!(output.status.success());
        assemble(std::str::from_utf8(&output.stdout).unwrap()).unwrap()
    })
}
fn execute(body: &str, extra: &str) -> Result<neoclr::Execution, neoclr::Fault> {
    let source = format!(
        r#".module Workers
.entry Main
.function Echo(String text) -> String
ldarg text
ret
.end
{extra}
.function Main() -> String
{body}
ret
.end
"#
    );
    let library = library();
    let app = neoclr::assembler::read_modules(
        &[neoclr::assembler::ModuleInput::Source(&source)],
        library,
    )?
    .remove(0);
    LoadedProgram::with_library(&app, library)?.run(Limits::default())
}

#[test]
fn dedicated_and_pooled_workers_exchange_owned_text() {
    for name in ["StartWorker", "QueueWorker"] {
        let result = execute(&format!("delegate.bind System.Func<String,String> = Echo(String)\nldstr \"👩‍💻\"\ncall neoCLR.Runtime.{name}(System.Func<String,String>,String)\ncall neoCLR.Runtime.JoinWorker(Int32)"), "").unwrap();
        assert_eq!(result.value, Value::String("👩‍💻".into()));
    }
}

#[test]
fn invalid_and_reused_handles_fault() {
    assert!(
        execute("ldc.i4 -1\ncall neoCLR.Runtime.JoinWorker(Int32)", "")
            .unwrap_err()
            .message
            .contains("Unknown")
    );
    let body = ".local Int32 handle\ndelegate.bind System.Func<String,String> = Echo(String)\nldstr \"ok\"\ncall neoCLR.Runtime.StartWorker(System.Func<String,String>,String)\nstloc handle\nldloc handle\ncall neoCLR.Runtime.JoinWorker(Int32)\npop\nldloc handle\ncall neoCLR.Runtime.JoinWorker(Int32)";
    assert!(
        execute(body, "")
            .unwrap_err()
            .message
            .contains("already joined")
    );
}

#[test]
fn worker_fault_reaches_join_and_nested_workers_are_rejected() {
    let extra = ".function Nested(String input) -> String\ndelegate.bind System.Func<String,String> = Echo(String)\nldarg input\ncall neoCLR.Runtime.StartWorker(System.Func<String,String>,String)\ncall neoCLR.Runtime.JoinWorker(Int32)\nret\n.end";
    let body = "delegate.bind System.Func<String,String> = Nested(String)\nldstr \"nested\"\ncall neoCLR.Runtime.QueueWorker(System.Func<String,String>,String)\ncall neoCLR.Runtime.JoinWorker(Int32)";
    assert!(
        execute(body, extra)
            .unwrap_err()
            .message
            .contains("Nested worker")
    );
}
