//! Standard byte channels below the Raven adapters.
use neoclr::{Console, ExecutionOptions, LoadedProgram, Value, assemble};
use std::{
    io,
    sync::{Arc, Mutex},
};

fn library() -> &'static neoclr::Module {
    static LIBRARY: std::sync::OnceLock<neoclr::Module> = std::sync::OnceLock::new();
    LIBRARY.get_or_init(|| {
        let output = std::process::Command::new("python3")
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .args(["-c", "import runpy; m=runpy.run_path('docs/experiments/raven-target/collection_library.py'); print(m['build'](m['ROOT'] / 'runtime/System.neoil'))"])
            .output().unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assemble(std::str::from_utf8(&output.stdout).unwrap()).unwrap()
    })
}
fn execute(body: &str, options: ExecutionOptions) -> neoclr::Execution {
    let source = format!(
        r#".module ConsoleStreams
.entry Main
.function Main() -> Int32
.local arrayref<Byte> buffer
ldc.i4 3
newarr Byte
stloc buffer
ldloc buffer
ldc.i4 0
ldc.i4 65
conv.u1
stelem Byte
ldloc buffer
ldc.i4 1
ldc.i4 255
conv.u1
stelem Byte
ldloc buffer
ldc.i4 2
ldc.i4 10
conv.u1
stelem Byte
{body}
ret
.end
"#
    );
    let app = neoclr::assembler::read_modules(
        &[neoclr::assembler::ModuleInput::Source(&source)],
        library(),
    )
    .unwrap()
    .remove(0);
    LoadedProgram::with_library(&app, library())
        .unwrap()
        .run(options)
        .unwrap()
}
fn write(error: bool, offset: i32, count: i32) -> String {
    format!(
        "ldc.bool {error}\nldloc buffer\nldc.i4 {offset}\nldc.i4 {count}\ncall neoCLR.Runtime.ConsoleWriteBytes(Boolean,arrayref<Byte>,Int32,Int32)\n"
    )
}
#[test]
fn capture_keeps_raw_channels_and_legacy_lines_separate() {
    let body = format!(
        "{}pop\n{}pop\nldstr \"line\"\ncall neoCLR.Runtime.WriteLine(String)\npop\nldc.bool true\ncall neoCLR.Runtime.ConsoleFlush(Boolean)\nvalue.unpack Int32",
        write(false, 0, 3),
        write(true, 1, 1)
    );
    let result = execute(&body, ExecutionOptions::default());
    assert_eq!(result.value, Value::Int32(0));
    assert_eq!(result.stdout, b"A\xff\nline\n");
    assert_eq!(result.stderr, b"\xff");
    assert_eq!(result.output, ["line"]);
}
#[test]
fn invalid_ranges_do_not_emit_and_zero_length_succeeds() {
    for (offset, count) in [(-1, 1), (0, -1), (3, 1), (4, 0)] {
        let result = execute(
            &(write(false, offset, count) + "value.unpack Byte\nconv.i4"),
            ExecutionOptions::default(),
        );
        assert_eq!(result.value, Value::Int32(7));
        assert!(result.stdout.is_empty() && result.stderr.is_empty());
    }
    assert_eq!(
        execute(
            &(write(false, 3, 0) + "value.unpack Int32"),
            ExecutionOptions::default()
        )
        .value,
        Value::Int32(0)
    );
}
#[derive(Debug, Default)]
struct ShortHost {
    calls: Mutex<Vec<(bool, Vec<u8>)>>,
}
impl Console for ShortHost {
    fn read_byte(&self) -> io::Result<Option<u8>> {
        Ok(None)
    }
    fn write_line(&self, _: &str) -> io::Result<()> {
        Err(io::Error::other("unexpected line write"))
    }
    fn write_bytes(&self, error: bool, bytes: &[u8]) -> io::Result<usize> {
        self.calls
            .lock()
            .unwrap()
            .push((error, bytes[..1].to_vec()));
        Ok(1)
    }
    fn flush(&self, _: bool) -> io::Result<()> {
        Err(io::Error::other("injected flush failure"))
    }
}
#[test]
fn host_short_write_and_flush_failure_are_preserved() {
    let host = Arc::new(ShortHost::default());
    let result = execute(
        &(write(true, 0, 3) + "value.unpack Int32"),
        ExecutionOptions {
            console: Some(host.clone()),
            ..Default::default()
        },
    );
    assert_eq!(result.value, Value::Int32(1));
    assert_eq!(*host.calls.lock().unwrap(), [(true, vec![65])]);
    assert!(result.stdout.is_empty() && result.stderr.is_empty());
    let failed = execute(
        "ldc.bool true\ncall neoCLR.Runtime.ConsoleFlush(Boolean)\nvalue.unpack Byte\nconv.i4",
        ExecutionOptions {
            console: Some(host),
            ..Default::default()
        },
    );
    assert_eq!(failed.value, Value::Int32(10));
}
