use neoclr::{Limits, Value, assemble, load, run, run_with_native};
use std::{
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Fixture {
    dir: PathBuf,
    import: String,
}
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "neoclr-native-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&dir).unwrap();
        let output = dir.join(libloading::library_filename("sample"));
        let status = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
            .args([
                "--edition=2024",
                "--crate-type=cdylib",
                "--crate-name=native_fixture",
            ])
            .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/native/library.rs"))
            .arg("-o")
            .arg(output)
            .status()
            .unwrap();
        assert!(status.success());
        let import = serde_json::to_string(&dir.join("sample").to_string_lossy()).unwrap();
        Self { dir, import }
    }
    fn module(&self, parameters: &str, returns: &str, symbol: &str, body: &str) -> neoclr::Module {
        assemble(&format!(".module Test\n.entry Main\n.function Imported({parameters}) -> {returns}\n.pinvoke {} {symbol:?} cdecl\n.end\n.function Main() -> {returns}\n{body}\ncall Imported({parameters})\nret\n.end", self.import)).unwrap()
    }
    fn execute(&self, module: &neoclr::Module) -> Result<neoclr::Execution, neoclr::Fault> {
        // SAFETY: test imports match the C-ABI fixture compiled above; tests only
        // pass live initialized storage, null, or scalars required by each export.
        unsafe {
            run_with_native(
                module,
                neoclr::library::system().unwrap(),
                Limits::default(),
            )
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn sample_round_trips_calls_native_and_frees_storage() {
    let fixture = Fixture::new();
    let source = include_str!("../examples/pinvoke.neoil")
        .replace("\"./target/native/neoclr_sample\"", &fixture.import);
    let module = assemble(&source).unwrap();
    let loaded = load(&serde_json::to_string(&module).unwrap()).unwrap();
    let import = loaded
        .functions
        .iter()
        .find(|f| f.name == "Native.Set")
        .unwrap();
    assert_eq!(
        import.pinvoke.as_ref().unwrap().entry_point,
        "neoclr_set_i32"
    );
    let execution = fixture.execute(&loaded).unwrap();
    assert_eq!(execution.output, ["42"]);
    assert_eq!(execution.value, Value::Void);
    assert_eq!(execution.memory.live_allocations(), 0);
    assert_eq!(
        execution.native_libraries.as_ref().unwrap().loaded_count(),
        1
    );
}

#[test]
fn scalar_arguments_and_returns_follow_the_c_abi() {
    let fixture = Fixture::new();
    for (ty, symbol, body, expected) in [
        ("SByte", "neoclr_i8", "ldc.i4 -128", Value::SByte(-128)),
        ("Byte", "neoclr_u8", "ldc.i4 255", Value::Byte(255)),
        ("Int16", "neoclr_i16", "ldc.i4 -32768", Value::Int16(-32768)),
        ("UInt16", "neoclr_u16", "ldc.i4 65535", Value::UInt16(65535)),
        (
            "Int32",
            "neoclr_i32",
            "ldc.i4 -2147483648",
            Value::Int32(i32::MIN),
        ),
        ("UInt32", "neoclr_u32", "ldc.i4 -1", Value::UInt32(u32::MAX)),
        (
            "Int64",
            "neoclr_i64",
            "ldc.i8 -9223372036854775808",
            Value::Int64(i64::MIN),
        ),
        ("UInt64", "neoclr_u64", "ldc.i8 -1", Value::UInt64(u64::MAX)),
        (
            "IntPtr",
            "neoclr_isize",
            "ldc.i4 -1\nconv.i",
            Value::IntPtr(-1),
        ),
        (
            "UIntPtr",
            "neoclr_usize",
            "ldc.i4 -1\nconv.i\nconv.u",
            Value::UIntPtr(usize::MAX),
        ),
        ("Single", "neoclr_f32", "ldc.r4 1.5", Value::Single(1.5)),
        ("Double", "neoclr_f64", "ldc.r8 -2.5", Value::Double(-2.5)),
    ] {
        assert_eq!(
            fixture
                .execute(&fixture.module(ty, ty, symbol, body))
                .unwrap()
                .value,
            expected,
            "{ty}"
        );
    }
    let mixed = fixture.module(
        "SByte,UInt16,Single,Double",
        "Double",
        "neoclr_mixed",
        "ldc.i4 -2\nldc.i4 40\nldc.r4 1.5\nldc.r8 2.5",
    );
    assert_eq!(fixture.execute(&mixed).unwrap().value, Value::Double(42.0));
}

#[test]
fn pointer_null_and_native_return_aliases_have_explicit_lifetimes() {
    let fixture = Fixture::new();
    let null = fixture.module("Int32*", "Int32*", "neoclr_identity_ptr", "ptr.null Int32");
    let execution = fixture.execute(&null).unwrap();
    let Value::Pointer(pointer) = &execution.value else {
        panic!()
    };
    assert_eq!(pointer.address, 0);
    let module = fixture.module(
        "Int32*",
        "Int32*",
        "neoclr_identity_ptr",
        "ldc.i4 1\nheap.alloc Int32\ndup\nldc.i4 42\nstind.i4",
    );
    let execution = fixture.execute(&module).unwrap();
    let Value::Pointer(pointer) = &execution.value else {
        panic!()
    };
    assert!(pointer.allocation.is_some());
    assert_eq!(execution.memory.live_allocations(), 1);
    // SAFETY: execution retains the live initialized allocation and native library.
    assert_eq!(unsafe { (pointer.address as *const i32).read() }, 42);
    let stale = fixture.module(
        "Int32*",
        "Int32*",
        "neoclr_identity_ptr",
        "ldc.i4 1\nheap.alloc Int32\ndup\nheap.free\npop",
    );
    assert!(
        fixture
            .execute(&stale)
            .unwrap_err()
            .message
            .contains("use after free")
    );
}

#[test]
fn loading_is_lazy_and_safe_embedding_requires_explicit_trust() {
    let source = ".module Test\n.entry Main\n.function Never() -> Void\n.pinvoke \"/no/such/neoclr/library\" \"missing\" cdecl\n.end\n.function Main() -> Void\nldvoid\nret\n.end";
    let module = assemble(source).unwrap();
    assert_eq!(run(&module, Limits::default()).unwrap().value, Value::Void);
    let source = source.replace("ldvoid\nret", "call Never()\nret");
    let module = load(&serde_json::to_string(&assemble(&source).unwrap()).unwrap()).unwrap();
    let fault = run(&module, Limits::default()).unwrap_err();
    assert!(fault.message.contains("run_with_native"));
    assert_eq!(fault.function.as_deref(), Some("Main"));
}

#[test]
fn loader_and_symbol_failures_are_instruction_located_faults() {
    let fixture = Fixture::new();
    let missing = fixture.module("Int32", "Int32", "symbol_that_does_not_exist", "ldc.i4 1");
    let fault = fixture.execute(&missing).unwrap_err();
    assert!(fault.message.contains("cannot resolve native entry"));
    assert_eq!(fault.function.as_deref(), Some("Main"));
    let mut missing = missing;
    missing.functions[0].pinvoke.as_mut().unwrap().library =
        fixture.dir.join("missing").to_string_lossy().into();
    let fault = fixture.execute(&missing).unwrap_err();
    assert!(fault.message.contains("cannot load native library"));
    assert!(fault.instruction.is_some());
}

#[test]
fn import_metadata_rejects_unsupported_shapes_without_loading() {
    for declaration in [
        ".function F(Void) -> Void\n.pinvoke \"missing\" \"f\" cdecl\n.end",
        ".function F(String) -> Void\n.pinvoke \"missing\" \"f\" cdecl\n.end",
        ".function F() -> Boolean\n.pinvoke \"missing\" \"f\" cdecl\n.end",
        ".function F() -> Void\n.pinvoke \"missing\" \"f\" cdecl\nldvoid\nret\n.end",
        ".function F() -> Void\n.local Int32\n.pinvoke \"missing\" \"f\" cdecl\n.end",
        ".function F() -> Void\n.methodimpl InternalCall\n.pinvoke \"missing\" \"f\" cdecl\n.end",
        ".function F() -> Void\n.pinvoke \"\" \"f\" cdecl\n.end",
        ".function F() -> Void\n.pinvoke \"missing\" \"f\" stdcall\n.end",
        ".function F() -> Void\n.pinvoke \"missing\" \"f\" cdecl\n.pinvoke \"missing\" \"f\" cdecl\n.end",
        ".type T\n.method instance F() -> Void\n.pinvoke \"missing\" \"f\" cdecl\n.end\n.end",
    ] {
        assert!(
            assemble(&format!(".module Test\n{declaration}")).is_err(),
            "{declaration}"
        );
    }
    assert!(
        assemble(
            ".module Test\n.entry F\n.function F() -> Void\n.pinvoke \"missing\" \"f\" cdecl\n.end"
        )
        .is_err()
    );
    let module =
        assemble(".module Test\n.function F() -> Void\n.pinvoke \"missing\" \"f\" cdecl\n.end")
            .unwrap();
    let mut json = serde_json::to_value(&module).unwrap();
    json["functions"][0]["pinvoke"]["calling_convention"] = "Stdcall".into();
    assert!(load(&json.to_string()).is_err());
}

#[test]
fn foreign_pointers_can_be_returned_and_forwarded_without_claiming_vm_ownership() {
    let fixture = Fixture::new();
    let source = format!(
        ".module Test\n.entry Main\n.function Address() -> Int32*\n.pinvoke {} \"neoclr_static_ptr\" cdecl\n.end\n.function Read(Int32*) -> Int32\n.pinvoke {} \"neoclr_read_i32\" cdecl\n.end\n.function Main() -> Int32\ncall Address()\ncall Read(Int32*)\nret\n.end",
        fixture.import, fixture.import
    );
    let module = assemble(&source).unwrap();
    assert_eq!(fixture.execute(&module).unwrap().value, Value::Int32(73));
    let direct = assemble(&source.replace("call Read(Int32*)", "ldind.i4")).unwrap();
    assert!(
        fixture
            .execute(&direct)
            .unwrap_err()
            .message
            .contains("untracked native pointer")
    );
}
