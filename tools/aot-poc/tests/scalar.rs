use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "neoclr-aot-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn compile(source: &str, output: &Path) -> std::process::Output {
    let input = output.with_extension("neoil");
    fs::write(&input, source).unwrap();
    Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc"))
        .arg(input)
        .arg("Calculate")
        .arg(output)
        .output()
        .unwrap()
}
const SOURCE: &str = include_str!("../../../docs/experiments/aot-scalar/scalar.neoil");

#[test]
fn unsupported_or_invalid_programs_never_emit_an_object() {
    let cases = [
        (
            SOURCE.replace("    mul", "    div"),
            "unsupported instruction",
        ),
        (
            SOURCE.replace("    mul", "    add.ovf"),
            "unsupported instruction",
        ),
        (
            SOURCE.replace("    ldarg value\n    ldc.i4 2", "    ldc.i4 2"),
            "stack underflow",
        ),
        (
            SOURCE.replace("    call Twice(Int32)", "    call Calculate(Int32)"),
            "recursive calls",
        ),
        (
            SOURCE.replace("    call Twice(Int32)", "    call Missing(Int32)"),
            "unknown function overload Missing",
        ),
        (
            SOURCE.replace("    call Twice(Int32)", "    call Twice(Int32, Int32)"),
            "unknown function overload Twice",
        ),
        (
            SOURCE.replace("    mul", "    mul\n    ret\n    ldc.i4 0"),
            "invalid return",
        ),
        (
            SOURCE.replace("    ldarg value", "    ldarg 99"),
            "argument index outside signature",
        ),
        (
            SOURCE.replace("    mul", "    pop\n    pop"),
            "invalid return",
        ),
        (
            SOURCE.replace(
                ".function Twice(Int32 value) -> Int32",
                ".function Twice(Int32 value) -> Int64",
            ),
            "scalar profile",
        ),
        (
            format!("{SOURCE}\n.function Unused() -> Int32\nldc.i4 1\nldc.i4 0\ndiv\nret\n.end\n"),
            "unsupported instruction",
        ),
    ];
    for (source, expected) in cases {
        let temp = Temp::new();
        let path = temp.0.join("rejected.o");
        let result = compile(&source, &path);
        assert!(!result.status.success());
        let error = String::from_utf8_lossy(&result.stderr);
        assert!(
            error.contains(expected),
            "expected {expected:?}, got {error}"
        );
        assert!(!path.exists());
    }
}

#[test]
fn emits_macho_and_refuses_to_overwrite() {
    let temp = Temp::new();
    let path = temp.0.join("scalar.o");
    let result = compile(SOURCE, &path);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let object = fs::read(&path).unwrap();
    assert_eq!(&object[..4], &[0xcf, 0xfa, 0xed, 0xfe]); // little-endian Mach-O 64
    assert_eq!(
        u32::from_le_bytes(object[4..8].try_into().unwrap()),
        0x0100000c
    ); // ARM64
    assert!(!compile(SOURCE, &path).status.success());
    assert_eq!(fs::read(&path).unwrap(), object);
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_c_consumer_matches_interpreter_for_runtime_inputs() {
    let temp = Temp::new();
    let object = temp.0.join("scalar.o");
    let result = compile(SOURCE, &object);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let binary = temp.0.join("scalar");
    let host =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-scalar/host.c");
    let result = Command::new("clang")
        .args(["-arch", "arm64", "-Wall", "-Wextra", "-Werror"])
        .arg(host)
        .arg(&object)
        .arg("-o")
        .arg(&binary)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let symbols = Command::new("nm").arg("-u").arg(&object).output().unwrap();
    assert!(symbols.status.success());
    assert!(
        symbols.stdout.is_empty(),
        "scalar object must not import interpreter/runtime helpers"
    );
    let program = neoclr::LoadedProgram::new(&neoclr::assemble(SOURCE).unwrap()).unwrap();
    let function = program
        .resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap())
        .unwrap();
    for input in [
        i32::MIN,
        i32::MIN + 1,
        -100,
        -1,
        0,
        1,
        20,
        100,
        i32::MAX / 2,
        i32::MAX,
    ] {
        let interpreted = function
            .invoke(vec![neoclr::Value::Int32(input)], neoclr::Limits::default())
            .unwrap()
            .value;
        let neoclr::Value::Int32(expected) = interpreted else {
            panic!("unexpected interpreter result")
        };
        assert_eq!(expected, input.wrapping_mul(2).wrapping_add(2));
        let result = Command::new(&binary)
            .arg(input.to_string())
            .output()
            .unwrap();
        assert!(result.status.success());
        let actual: i32 = String::from_utf8(result.stdout)
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        assert_eq!(actual, expected, "input {input}");
    }
    for input in ["2147483648", "abc", ""] {
        assert_eq!(
            Command::new(&binary)
                .arg(input)
                .output()
                .unwrap()
                .status
                .code(),
            Some(2)
        );
    }
}
