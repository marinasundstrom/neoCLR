use neoclr::{Limits, LoadedProgram, Value, assemble, assembler::parse_function_ref};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::atomic::{AtomicU64, Ordering},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "neoclr-preview-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn cli(args: &[&str], input: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_neoclr"))
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}
fn successful(output: &Output) {
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone())
        .unwrap()
        .replace("\r\n", "\n")
}

#[test]
fn walkthrough_sources_and_artifacts_have_the_documented_outputs_and_exits() {
    let fixture = Fixture::new();
    let prompt = "Enter an unsigned number (up to nine digits):\n";
    type Scenario = (&'static str, Vec<(&'static [u8], String)>);
    let samples: Vec<Scenario> = vec![
        (
            "hello_functions",
            vec![(b"", "Hello, world!\n=> Void\n".into())],
        ),
        (
            "array_loops",
            vec![(b"", "Sum of squares:\n30\n=> Void\n".into())],
        ),
        (
            "file_summary",
            vec![(
                b"",
                "File contents:\nHello, neoCLR 🌍!\nUTF-8 bytes:\n19\n=> Void\n".into(),
            )],
        ),
        ("pointer_union", vec![(b"", "42\n7\n11\n=> Void\n".into())]),
        (
            "ordinary_unions",
            vec![(
                b"",
                "success\n7\nfailure\n7\nSome<Void> is present\n=> Void\n".into(),
            )],
        ),
        (
            "console_input",
            vec![
                (b"21\n", format!("{prompt}42\n=> Void\n")),
                (b"", format!("{prompt}End of input\n=> Void\n")),
                (b"\n", format!("{prompt}Empty input\n=> Void\n")),
                (b"x\n", format!("{prompt}Expected ASCII digits\n=> Void\n")),
                (
                    b"1234567890\n",
                    format!("{prompt}At most nine digits\n=> Void\n"),
                ),
            ],
        ),
        (
            "type_inspection",
            vec![(
                b"",
                "System.Int32\nBox\n1\nSystem.Int32\nSame type\n=> Void\n".into(),
            )],
        ),
        (
            "array_list",
            vec![(b"", "ArrayList count:\n5\n0\n1\n4\n9\n16\n=> Void\n".into())],
        ),
        ("interfaces", vec![(b"", "42\n2\n=> Void\n".into())]),
        ("array_bounds", vec![]),
        ("fault_trace", vec![]),
    ];
    for (name, cases) in samples {
        let source = format!("examples/{name}.neoil");
        let artifact = fixture.0.join(format!("{name}.neo.json"));
        let artifact = artifact.to_str().unwrap();
        successful(&cli(&["assemble", &source, artifact], b""));
        for path in [&*source, artifact] {
            successful(&cli(&["check", path], b""));
            successful(&cli(&["verify", path], b""));
            if matches!(name, "array_bounds" | "fault_trace") {
                let result = cli(&["run", path], b"");
                assert_eq!(result.status.code(), Some(1));
                let error = String::from_utf8_lossy(&result.stderr);
                assert!(error.contains("Fault:"), "{error}");
                if name == "array_bounds" {
                    assert!(error.contains("System.Array.GetElementAddress"), "{error}");
                } else {
                    assert!(error.contains("Demonstration fault"), "{error}");
                    let fail = error.find("  at Fail()").unwrap();
                    let work = error.find("  at Work()").unwrap();
                    let main = error.find("  at Main()").unwrap();
                    assert!(fail < work && work < main, "{error}");
                }
                assert!(error.contains("Main"), "{error}");
                assert!(error.contains("IL instruction"), "{error}");
                assert!(!stdout(&result).contains("=>"));
            } else {
                for (input, expected) in &cases {
                    let result = cli(&["run", path], input);
                    successful(&result);
                    assert_eq!(&stdout(&result), expected, "{path}");
                    assert!(result.stderr.is_empty());
                }
            }
        }
    }
}

#[test]
fn foreach_lowering_handles_empty_and_single_element_buffers_and_releases_storage() {
    for (length, expected) in [(0, 0), (1, 7), (5, 35)] {
        let source = format!(
            "{}\n.function Probe() -> Int32\n.local System.Array<Int32> values\nldc.i4 {length}\nldc.i4 7\ncall System.Array<Int32>::Allocate(Int32,Int32)\nstloc values\nldloc values\ncall Sum(System.Array<Int32>)\nldloc values\ncall instance System.Array<Int32>::Free()\npop\nret\n.end",
            include_str!("../examples/array_loops.neoil").replace(".entry Main", ".entry Probe")
        );
        let program = LoadedProgram::new(&assemble(&source).unwrap()).unwrap();
        program.verify().unwrap();
        let result = program.run(Limits::default()).unwrap();
        assert_eq!(result.value, Value::Int32(expected));
        assert_eq!(result.memory.live_allocations(), 0);
    }
}

#[test]
fn file_summary_reports_specific_errors_and_handles_empty_text() {
    let fixture = Fixture::new();
    let program =
        LoadedProgram::new(&assemble(include_str!("../examples/file_summary.neoil")).unwrap())
            .unwrap();
    program.verify().unwrap();
    let report = program
        .resolve_function(&parse_function_ref("Report(String)").unwrap())
        .unwrap();
    fs::write(fixture.0.join("invalid"), [0xff]).unwrap();
    fs::write(fixture.0.join("large"), vec![b'x'; 4097]).unwrap();
    fs::write(fixture.0.join("empty"), b"").unwrap();
    for (name, expected) in [
        ("missing", vec!["File not found"]),
        ("invalid", vec!["InvalidUtf8"]),
        ("large", vec!["FileTooLarge"]),
        ("empty", vec!["File contents:", "", "UTF-8 bytes:", "0"]),
    ] {
        let result = report
            .invoke(
                vec![Value::String(
                    fixture.0.join(Path::new(name)).to_str().unwrap().into(),
                )],
                Limits::default(),
            )
            .unwrap();
        assert_eq!(result.value, Value::Void);
        assert_eq!(result.output, expected);
    }
}
