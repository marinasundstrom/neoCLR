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
    compile_with_console(source, output, false)
}
fn compile_with_console(source: &str, output: &Path, console: bool) -> std::process::Output {
    let input = output.with_extension("neoil");
    fs::write(&input, source).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc"));
    command.arg(input).arg("Calculate").arg(output);
    if console {
        command.arg("--console");
    }
    command.output().unwrap()
}
const SOURCE: &str = include_str!("../../../docs/experiments/aot-scalar/scalar.neoil");

#[test]
fn unsupported_or_invalid_programs_never_emit_an_object() {
    let cases = [
        (
            SOURCE.replace("    mul", "    xor"),
            "unsupported instruction",
        ),
        (
            SOURCE.replace("    mul", "    shl"),
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
            SOURCE.replace("    mul", "    mul\n    ldc.i4 0"),
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
            format!("{SOURCE}\n.function Unused() -> Int32\nldc.i4 1\nldc.i4 0\nxor\nret\n.end\n"),
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
    assert_native_parity(
        SOURCE,
        &[
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
        ],
        |input| input.wrapping_mul(2).wrapping_add(2),
    );
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn assert_native_parity(source: &str, inputs: &[i32], oracle: impl Fn(i32) -> i32) {
    assert_native_outcomes(source, inputs, Some(&oracle));
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn assert_native_outcomes(source: &str, inputs: &[i32], oracle: Option<&dyn Fn(i32) -> i32>) {
    let temp = Temp::new();
    let object = temp.0.join("scalar.o");
    let result = compile(source, &object);
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
    let exported = Command::new("nm").arg("-g").arg(&object).output().unwrap();
    assert!(exported.status.success());
    let symbols = String::from_utf8(exported.stdout).unwrap();
    assert!(
        symbols
            .lines()
            .any(|line| line.ends_with(" _neoclr_entry_v2"))
    );
    assert!(!symbols.lines().any(|line| line.ends_with(" _neoclr_entry")));
    let program = neoclr::LoadedProgram::new(&neoclr::assemble(source).unwrap()).unwrap();
    let function = program
        .resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap())
        .unwrap();
    for &input in inputs {
        let interpreted =
            function.invoke(vec![neoclr::Value::Int32(input)], neoclr::Limits::default());
        let result = Command::new(&binary)
            .arg(input.to_string())
            .output()
            .unwrap();
        match interpreted {
            Ok(execution) => {
                let neoclr::Value::Int32(expected) = execution.value else {
                    panic!("unexpected interpreter result")
                };
                if let Some(oracle) = oracle {
                    assert_eq!(expected, oracle(input));
                }
                assert!(
                    result.status.success(),
                    "input {input}: {}",
                    String::from_utf8_lossy(&result.stderr)
                );
                assert!(result.stderr.is_empty());
                let actual: i32 = String::from_utf8(result.stdout)
                    .unwrap()
                    .trim()
                    .parse()
                    .unwrap();
                assert_eq!(actual, expected, "input {input}");
            }
            Err(fault) => {
                assert!(oracle.is_none(), "unexpected interpreter fault {fault}");
                assert!(matches!(
                    fault.code,
                    neoclr::FaultCode::DivideByZero | neoclr::FaultCode::ArithmeticOverflow
                ));
                assert_eq!(
                    result.status.code(),
                    Some(1),
                    "input {input}: {}",
                    String::from_utf8_lossy(&result.stderr)
                );
                assert!(result.stdout.is_empty());
                assert_eq!(
                    String::from_utf8(result.stderr).unwrap().trim(),
                    fault.code.as_str(),
                    "input {input}"
                );
            }
        }
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

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_loops_locals_calls_and_early_returns_match_interpreter() {
    assert_native_parity(
        include_str!("../../../docs/experiments/aot-scalar/control-flow.neoil"),
        &[i32::MIN, -1, 0, 1, 2, 5, 20, 21, i32::MAX],
        |n| {
            if n < 0 {
                -1
            } else if n > 20 {
                -2
            } else {
                n * (n - 1)
            }
        },
    );
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_comparison_branches_preserve_signedness() {
    let cases: [(&str, fn(i32) -> bool); 10] = [
        ("beq", |x| x == 0),
        ("bne.un", |x| x != 0),
        ("bgt", |x| x > 0),
        ("bgt.un", |x| (x as u32) > 0),
        ("blt", |x| x < 0),
        ("blt.un", |_| false),
        ("bge", |x| x >= 0),
        ("bge.un", |_| true),
        ("ble", |x| x <= 0),
        ("ble.un", |x| (x as u32) <= 0),
    ];
    for (op, predicate) in cases {
        // Both paths merge a live stack value, rather than returning immediately.
        let source = format!(
            ".module Branch\n.function Calculate(Int32 value) -> Int32\nldarg value\nldc.i4 0\n{op} Taken\nldc.i4 222\nbr Join\nTaken:\nldc.i4 111\nJoin:\nldc.i4 1\nadd\nret\n.end\n"
        );
        assert_native_parity(&source, &[i32::MIN, -1, 0, 1, i32::MAX], |x| {
            if predicate(x) { 112 } else { 223 }
        });
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_truth_branches_merge_locals_and_loop_carried_stack() {
    for op in ["brtrue", "brfalse"] {
        let source = format!(
            ".module Merge\n.function Calculate(Int32 value) -> Int32\n.local Int32 count\nldarg value\n{op} Taken\nldc.i4 2\nstloc count\nbr Join\nTaken:\nldc.i4 3\nstloc count\nJoin:\nldc.i4 10\nLoop:\nldc.i4 1\nadd\nldloc count\nldc.i4 1\nsub\nstloc count\nldloc count\nbrtrue Loop\nret\n.end\n"
        );
        assert_native_parity(&source, &[i32::MIN, -1, 0, 1, i32::MAX], |x| {
            if (x != 0) == (op == "brtrue") { 13 } else { 12 }
        });
    }
}

#[test]
fn invalid_control_flow_is_rejected_before_emission() {
    let cases = [
        (
            "ldarg 0\nbrtrue Join\nldc.i4 1\nJoin:\nldc.i4 2\nret",
            "incompatible stack heights",
        ),
        (
            ".local Int32 x\nldarg 0\nbrtrue Join\nldc.i4 1\nstloc x\nJoin:\nldloc x\nret",
            "not initialized",
        ),
        ("ldc.i4 1\nbr Missing\nret", "Missing"),
        (
            "ldarg 0\nbrtrue Start\nStart:\nldc.i4 1",
            "control flow leaves",
        ),
        (
            "ldc.i4 0\nLoop:\nldc.i4 1\nbr Loop",
            "incompatible stack heights",
        ),
        (
            "ldc.i4 1\nret\nldc.i4 1\nldc.i4 0\nxor\nret",
            "unsupported instruction",
        ),
    ];
    for (body, expected) in cases {
        let source = format!(
            ".module InvalidFlow\n.function Calculate(Int32 value) -> Int32\n{body}\n.end\n"
        );
        let temp = Temp::new();
        let path = temp.0.join("rejected.o");
        let result = compile(&source, &path);
        assert!(!result.status.success());
        assert!(
            String::from_utf8_lossy(&result.stderr).contains(expected),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(!path.exists());
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_arithmetic_faults_match_interpreter_and_preserve_output() {
    let inputs = [
        i32::MIN,
        i32::MIN + 1,
        -2,
        -1,
        0,
        1,
        2,
        i32::MAX / 2,
        i32::MAX,
    ];
    for (op, rhs) in [
        ("add.ovf", 1),
        ("sub.ovf", 1),
        ("mul.ovf", 2),
        ("add.ovf.un", 1),
        ("sub.ovf.un", 1),
        ("mul.ovf.un", 2),
    ] {
        // Two compiled call boundaries exercise status propagation to the C host.
        let source = format!(
            ".module Checked\n.function Operation(Int32 value) -> Int32\nldarg value\nldc.i4 {rhs}\n{op}\nret\n.end\n.function Forward(Int32 value) -> Int32\nldarg value\ncall Operation(Int32)\nret\n.end\n.function Calculate(Int32 value) -> Int32\nldarg value\ncall Forward(Int32)\nret\n.end\n"
        );
        assert_native_outcomes(&source, &inputs, None);
    }
    for op in ["div", "rem", "div.un", "rem.un"] {
        let source = format!(
            ".module Division\n.function Operation(Int32 value) -> Int32\nldc.i4 -2147483648\nldarg value\n{op}\nret\n.end\n.function Calculate(Int32 value) -> Int32\nldarg value\ncall Operation(Int32)\nret\n.end\n"
        );
        assert_native_outcomes(&source, &inputs, None);
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn first_fault_wins_across_compiled_calls() {
    assert_native_outcomes(
        include_str!("../../../docs/experiments/aot-scalar/faults.neoil"),
        &[i32::MAX, -1, 19],
        None,
    );
    let source = ".module FirstFault\n.function Overflow(Int32 value) -> Int32\nldarg value\nldc.i4 1\nadd.ovf\nret\n.end\n.function Calculate(Int32 value) -> Int32\nldarg value\ncall Overflow(Int32)\nldc.i4 0\ndiv\nret\n.end\n";
    // MAX faults in the callee; 0 gets through and faults in the caller.
    assert_native_outcomes(source, &[i32::MAX, 0], None);
}

const CONSOLE_SOURCE: &str = include_str!("../../../docs/experiments/aot-scalar/console.neoil");

#[test]
fn console_capability_and_literal_types_are_checked() {
    let temp = Temp::new();
    let output = temp.0.join("console.o");
    let rejected = compile(CONSOLE_SOURCE, &output);
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("--console"));
    assert!(!output.exists());
    for body in [
        "ldstr \"not an integer\"\nret",
        "ldc.i4 12\ncall neoCLR.Runtime.WriteLine(String)\npop\nldc.i4 0\nret",
        "ldarg 0\nbrtrue String\nldc.i4 0\nbr Join\nString:\nldstr \"text\"\nJoin:\npop\nldc.i4 0\nret",
    ] {
        let source = format!(
            ".module BadConsole\n.function Calculate(Int32 value) -> Int32\n{body}\n.end\n"
        );
        let result = compile_with_console(&source, &output, true);
        assert!(!result.status.success());
        assert!(!output.exists());
    }
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn native_console_preserves_utf8_bytes_and_propagates_service_failure() {
    let temp = Temp::new();
    let object = temp.0.join("console.o");
    let result = compile_with_console(CONSOLE_SOURCE, &object, true);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let imports = Command::new("nm").arg("-u").arg(&object).output().unwrap();
    assert!(imports.status.success());
    assert_eq!(
        String::from_utf8(imports.stdout).unwrap().trim(),
        "_neoclr_console_write_line_utf8_v1"
    );
    let sources = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/experiments/aot-scalar");
    let link = |service: Option<&Path>, binary: &Path| {
        let mut command = Command::new("clang");
        command
            .args(["-arch", "arm64", "-Wall", "-Wextra", "-Werror"])
            .arg(sources.join("host.c"))
            .arg(&object)
            .arg("-o")
            .arg(binary);
        if let Some(service) = service {
            command.arg(service);
        }
        command.output().unwrap()
    };
    let binary = temp.0.join("console");
    assert!(
        !link(None, &binary).status.success(),
        "missing console service must fail linking"
    );
    let result = link(Some(&sources.join("console.c")), &binary);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let dependencies = Command::new("otool")
        .arg("-L")
        .arg(&binary)
        .output()
        .unwrap();
    assert!(dependencies.status.success());
    let listing = String::from_utf8(dependencies.stdout).unwrap();
    let dependencies: Vec<_> = listing
        .lines()
        .skip(1)
        .map(|line| line.split_whitespace().next().unwrap())
        .collect();
    assert_eq!(dependencies, ["/usr/lib/libSystem.B.dylib"]);
    // Execute only the linked binary from a fresh directory and empty environment.
    // No IL, object, compiler, service library or framework accompanies it.
    let isolated = Temp::new();
    let binary_copy = isolated.0.join("app");
    fs::copy(&binary, &binary_copy).unwrap();
    let program = neoclr::LoadedProgram::new(&neoclr::assemble(CONSOLE_SOURCE).unwrap()).unwrap();
    let function = program
        .resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap())
        .unwrap();
    for value in [0, -1, 42] {
        let execution = function
            .invoke(vec![neoclr::Value::Int32(value)], neoclr::Limits::default())
            .unwrap();
        let mut expected = Vec::new();
        for line in execution.output {
            expected.extend_from_slice(line.as_bytes());
            expected.push(b'\n');
        }
        expected.extend_from_slice(format!("{value}\n").as_bytes());
        let actual = Command::new(&binary_copy)
            .arg(value.to_string())
            .current_dir(&isolated.0)
            .env_clear()
            .output()
            .unwrap();
        assert!(actual.status.success());
        assert!(actual.stderr.is_empty());
        assert_eq!(actual.stdout, expected);
    }
    // A deliberately failing service records only its first invocation. Native
    // fault propagation must skip every later write and preserve the root output.
    let fail_service = temp.0.join("fail.c");
    fs::write(&fail_service, "#include <stdint.h>\n#include <stddef.h>\n#include <stdio.h>\nint32_t neoclr_console_write_line_utf8_v1(const uint8_t *p, size_t n) { (void)p; (void)n; fputs(\"called\\n\", stderr); return 99; }\n").unwrap();
    let failed_binary = temp.0.join("failure");
    let result = link(Some(&fail_service), &failed_binary);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let actual = Command::new(&failed_binary).arg("42").output().unwrap();
    assert_eq!(actual.status.code(), Some(1));
    assert!(actual.stdout.is_empty());
    assert_eq!(actual.stderr, b"called\nRuntimeError\n");
    // Exercise the actual implementation's flush error, not only a stub status.
    let closed_host = temp.0.join("closed.c");
    fs::write(&closed_host, "#include \"abi.h\"\n#include <unistd.h>\nint main(void) { int32_t value = 99; close(STDOUT_FILENO); int32_t status = neoclr_entry_v2(42, &value); return status == NEOCLR_AOT_RUNTIME_ERROR && value == 99 ? 0 : 1; }\n").unwrap();
    let closed = temp.0.join("closed");
    let result = Command::new("clang")
        .args(["-arch", "arm64", "-Wall", "-Wextra", "-Werror"])
        .arg("-I")
        .arg(&sources)
        .arg(&closed_host)
        .arg(sources.join("console.c"))
        .arg(&object)
        .arg("-o")
        .arg(&closed)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(Command::new(&closed).output().unwrap().status.success());
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn standalone_hello_world_has_no_managed_runtime_dependency() {
    let temp = Temp::new();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    assert_standalone_hello(
        &temp,
        &root.join("docs/experiments/aot-hello/hello.neoil"),
        "Main",
    );
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn assert_standalone_hello(temp: &Temp, input: &Path, entry: &str) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let object = temp.0.join("hello.o");
    let result = Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc"))
        .arg(input)
        .arg(entry)
        .arg(&object)
        .arg("--console")
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let executable = temp.0.join("hello");
    let result = Command::new("clang")
        .args(["-arch", "arm64", "-Wall", "-Wextra", "-Werror"])
        .arg(root.join("docs/experiments/aot-hello/main.c"))
        .arg(root.join("docs/experiments/aot-scalar/console.c"))
        .arg(&object)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let imports = Command::new("otool")
        .arg("-L")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(imports.status.success());
    let listing = String::from_utf8(imports.stdout).unwrap();
    let imports: Vec<_> = listing
        .lines()
        .skip(1)
        .map(|line| line.split_whitespace().next().unwrap())
        .collect();
    assert_eq!(imports, ["/usr/lib/libSystem.B.dylib"]);
    let isolated = Temp::new();
    let installed = isolated.0.join("hello");
    fs::copy(&executable, &installed).unwrap();
    let result = Command::new(&installed)
        .env_clear()
        .current_dir(&isolated.0)
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(0));
    assert_eq!(result.stdout, b"Hello, world!\n");
    assert!(result.stderr.is_empty());
}

const RAVEN_HELLO: &[u8] = include_bytes!("../../../docs/experiments/aot-hello/RavenHello.pe");

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn raven_native_metadata_hello_runs_as_standalone_machine_code() {
    let module = neoclr::metadata_container::decode(RAVEN_HELLO).unwrap();
    assert_eq!(module.functions.len(), 1);
    assert_eq!(module.functions[0].origin.as_ref().unwrap().name, "Main");
    assert!(module.functions[0].parameters.is_empty());
    let program = neoclr::LoadedProgram::new(&module).unwrap();
    let interpreted = program.run(neoclr::ExecutionOptions::default()).unwrap();
    assert_eq!(interpreted.output, ["Hello, world!"]);
    for bytes in [
        RAVEN_HELLO.to_vec(),
        neoclr::metadata_container::write_module(&module).unwrap(),
    ] {
        let temp = Temp::new();
        let input = temp.0.join("raven.bin"); // Recognition uses magic, not the extension.
        fs::write(&input, bytes).unwrap();
        assert_standalone_hello(&temp, &input, "@entry");
    }
}

#[test]
fn native_input_rejects_corruption_and_unsupported_il_before_emission() {
    use neoclr::metadata::Instruction as Op;
    let module = neoclr::metadata_container::decode(RAVEN_HELLO).unwrap();
    let envelope = neoclr::metadata_container::write_module(&module).unwrap();
    let mut wrong_marker = RAVEN_HELLO.to_vec();
    let marker = wrong_marker
        .windows(11)
        .position(|w| w == b"neoCLR.NEOX")
        .unwrap();
    wrong_marker[marker] = b'x';
    let mut corrupt_pe = RAVEN_HELLO.to_vec();
    let payload = corrupt_pe.windows(4).position(|w| w == b"NEOX").unwrap();
    corrupt_pe[payload + 40] ^= 1;
    let mut unsupported = module.clone();
    unsupported.functions[0].body.insert(0, Op::BitXor);
    let mut invalid = module.clone();
    invalid.functions[0].body.remove(0); // Console call without its String argument.
    let mut wrong_owner = module.clone();
    if let Op::Call(call) = &mut wrong_owner.functions[0].body[1] {
        call.owner = None;
    }
    let mut no_entry = module.clone();
    no_entry.entry.clear();
    let mut cases = vec![
        (wrong_marker, "recognition marker"),
        (corrupt_pe, "binding mismatch"),
        (
            envelope[..envelope.len() - 1].to_vec(),
            "native metadata container",
        ),
        (
            neoclr::metadata_container::write_module(&unsupported).unwrap(),
            "unsupported instruction",
        ),
        (
            neoclr::metadata_container::write_module(&invalid).unwrap(),
            "stack underflow",
        ),
        (
            neoclr::metadata_container::write_module(&wrong_owner).unwrap(),
            "unsupported operand type",
        ),
        (
            neoclr::metadata_container::write_module(&no_entry).unwrap(),
            "root function not found",
        ),
    ];
    let mut trailing = envelope.clone();
    trailing.push(0);
    cases.push((trailing, "native metadata container"));
    for (bytes, expected) in cases {
        let temp = Temp::new();
        let input = temp.0.join("input.neox");
        let output = temp.0.join("rejected.o");
        fs::write(&input, bytes).unwrap();
        let result = Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc"))
            .arg(&input)
            .arg("@entry")
            .arg(&output)
            .arg("--console")
            .output()
            .unwrap();
        assert!(!result.status.success());
        assert!(
            String::from_utf8_lossy(&result.stderr).contains(expected),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(!output.exists());
    }
    let temp = Temp::new();
    let input = temp.0.join("input.neox");
    let output = temp.0.join("preserved.o");
    fs::write(&input, envelope).unwrap();
    // Capability policy applies to binary inputs as well as text.
    let result = Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc"))
        .arg(&input)
        .arg("@entry")
        .arg(&output)
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("--console"));
    assert!(!output.exists());
    fs::write(&output, b"existing artifact").unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc"))
        .arg(&input)
        .arg("@entry")
        .arg(&output)
        .arg("--console")
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert_eq!(fs::read(output).unwrap(), b"existing artifact");
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn parameterless_entry_adapter_preserves_fault_and_result_pointer() {
    let mut module = neoclr::metadata_container::decode(RAVEN_HELLO).unwrap();
    use neoclr::metadata::Instruction as Op;
    module.functions[0].body = vec![Op::Int(1), Op::Int(0), Op::Divide, Op::Return];
    let temp = Temp::new();
    let input = temp.0.join("fault.neox");
    let object = temp.0.join("fault.o");
    fs::write(
        &input,
        neoclr::metadata_container::write_module(&module).unwrap(),
    )
    .unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc"))
        .arg(&input)
        .arg("@entry")
        .arg(&object)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let host = temp.0.join("host.c");
    fs::write(
        &host,
        r#"
#include <stdint.h>
extern int32_t neoclr_entry_v2(int32_t, int32_t *);
int main(void) {
    int32_t result = 12345;
    int32_t status = neoclr_entry_v2(99, &result);
    return status == 1 && result == 12345 ? 0 : 1;
}
"#,
    )
    .unwrap();
    let executable = temp.0.join("fault");
    let result = Command::new("clang")
        .args(["-arch", "arm64", "-Wall", "-Wextra", "-Werror"])
        .arg(&host)
        .arg(&object)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(Command::new(executable).output().unwrap().status.success());
}
