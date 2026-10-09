use object::{Object, ObjectSection, ObjectSymbol};
use std::{fs, path::PathBuf, process::Command, sync::atomic::{AtomicUsize, Ordering}};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("neoclr-windows-stack-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }
const SOURCE: &str = include_str!("../../../tools/native/windows-generated-stack.neoil");
const FLAGS: &[&str] = &["--target", "x86_64-pc-windows-msvc", "--windows-stack-experiment", "--reference-arena", "--native-gc", "--native-stack-budget"];
fn compile(dir: &Temp, source: &str, flags: &[&str]) -> std::process::Output {
    fs::write(dir.0.join("input.neoil"), source).unwrap();
    Command::new(env!("CARGO_BIN_EXE_neoclr-aot-poc"))
        .arg(dir.0.join("input.neoil")).arg("Calculate").arg(dir.0.join("app.obj"))
        .args(flags).output().unwrap()
}
#[test]
fn recursive_windows_object_has_inline_page_probe_and_context_entry() {
    let dir = Temp::new();
    let result = compile(&dir, SOURCE, FLAGS);
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let bytes = fs::read(dir.0.join("app.obj")).unwrap();
    let obj = object::File::parse(bytes.as_slice()).unwrap();
    assert_eq!(obj.format(), object::BinaryFormat::Coff);
    assert_eq!(obj.architecture(), object::Architecture::X86_64);
    let names: Vec<_> = obj.symbols().filter_map(|s| s.name().ok()).collect();
    for name in ["neoclr_entry_v4", "neoclr_native_stack_check_v1", "neoclr_gc_enter_v1", "neoclr_gc_leave_v1", "neoclr_gc_stack_roots_v1"] {
        assert!(names.contains(&name), "missing {name}");
    }
    // Pinned Cranelift 0.121.2: sub rsp,4096; mov [rsp],esp; add rsp,4096.
    // Checking emitted bytes catches a disabled setting or outline-only helper.
    let probe = [0x48,0x81,0xec,0x00,0x10,0x00,0x00,0x89,0x24,0x24,0x48,0x81,0xc4,0x00,0x10,0x00,0x00];
    let code = obj.section_by_name(".text").unwrap().data().unwrap();
    assert!(code.windows(probe.len()).any(|w| w == probe));
    assert!(!names.iter().any(|n| n.contains("chkstk") || n.contains("probestack")));
}
#[test]
fn windows_stack_requires_explicit_complete_opt_in_and_rejects_managed_types() {
    for flags in [
        FLAGS.iter().copied().filter(|f| *f != "--windows-stack-experiment").collect::<Vec<_>>(),
        FLAGS.iter().copied().filter(|f| *f != "--native-stack-budget").collect(),
        FLAGS.iter().copied().filter(|f| *f != "--native-gc").collect(),
        FLAGS.iter().copied().filter(|f| *f != "--reference-arena").collect(),
        FLAGS.iter().map(|f| if *f == "x86_64-pc-windows-msvc" { "aarch64-apple-darwin" } else { *f }).collect(),
    ] {
        let dir = Temp::new();
        let result = compile(&dir, SOURCE, &flags);
        assert!(!result.status.success());
        assert!(!dir.0.join("app.obj").exists());
    }
    for source in [SOURCE.replace(".local Int32 pad0", ".local String pad0"), SOURCE.replace("ldc.i4 42", "ldstr \"no service admission\"\npop\nldc.i4 42")] {
        let dir = Temp::new();
        let result = compile(&dir, &source, FLAGS);
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains("only Int32 functions"));
        assert!(!dir.0.join("app.obj").exists());
    }
}

#[test]
fn heap_experiment_admits_only_int32_arrays_and_matches_interpreter() {
    let source = include_str!("../../../tools/native/windows-generated-heap.neoil");
    let flags: Vec<_> = FLAGS.iter().map(|f| if *f == "--windows-stack-experiment" { "--windows-heap-experiment" } else { *f }).collect();
    let dir = Temp::new();
    let result = compile(&dir, source, &flags);
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let app = neoclr::assemble(source).unwrap();
    let program = neoclr::LoadedProgram::new(&app).unwrap();
    let method = program.resolve_function(&neoclr::assembler::parse_function_ref("Calculate(Int32)").unwrap()).unwrap();
    assert_eq!(method.invoke(vec![neoclr::Value::Int32(0)], neoclr::Limits::default()).unwrap().value, neoclr::Value::Int32(42));
    assert!(method.invoke(vec![neoclr::Value::Int32(1)], neoclr::Limits::default()).is_err());
    for (input, flags) in [(source.to_owned(), FLAGS.to_vec()),
        (source.replace("arrayref<Int32>", "arrayref<String>"), flags.clone()),
        (source.replace("newarr Int32", "newarr Byte"), flags)] {
        let dir = Temp::new();
        let result = compile(&dir, &input, &flags);
        assert!(!result.status.success());
        assert!(!dir.0.join("app.obj").exists());
    }
}
