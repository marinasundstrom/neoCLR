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

#[test]
fn explicit_input_strings_preserve_encoding_and_call_scoped_buffers() {
    let fixture = Fixture::new();
    let sample = include_str!("../examples/foreign_strings.neoil")
        .replace("\"./target/native/neoclr_sample\"", &fixture.import);
    assert_eq!(
        fixture.execute(&assemble(&sample).unwrap()).unwrap().output,
        ["7", "4"]
    );
    for (encoding, symbol, text, length) in [
        ("utf8", "neoclr_utf8_len", "", 0),
        ("utf8", "neoclr_utf8_len", "hé🦀", 7),
        ("utf16", "neoclr_utf16_len", "", 0),
        ("utf16", "neoclr_utf16_len", "hé🦀", 4),
    ] {
        let source = format!(
            ".module Test\n.entry Main\n.function Length(String) -> UIntPtr\n.foreign {} {symbol:?} cdecl {encoding}\n.end\n.function Main() -> UIntPtr\nldstr {text:?}\ncall Length(String)\nret\n.end",
            fixture.import
        );
        let module = assemble(&source).unwrap();
        let module = load(&serde_json::to_string(&module).unwrap()).unwrap();
        assert_eq!(
            fixture.execute(&module).unwrap().value,
            Value::UIntPtr(length)
        );
    }
    let source = format!(
        ".module Test\n.entry Main\n.function Pair(String,Int32,String) -> Int32\n.foreign {} \"neoclr_utf8_pair\" cdecl utf8\n.end\n.function Main() -> Int32\nldstr \"hé🦀\"\nldc.i4 41\nldstr \"世界\"\ncall Pair(String,Int32,String)\npop\nldstr \"hé🦀\"\nldc.i4 99\nldstr \"世界\"\ncall Pair(String,Int32,String)\nret\n.end",
        fixture.import
    );
    assert_eq!(
        fixture.execute(&assemble(&source).unwrap()).unwrap().value,
        Value::Int32(100)
    );
    let source = format!(
        ".module Test\n.entry Main\n.function Check(String) -> Int32\n.foreign {} \"neoclr_utf16_check\" cdecl utf16\n.end\n.function Main() -> Int32\nldstr \"hé🦀\"\ncall Check(String)\nret\n.end",
        fixture.import
    );
    assert_eq!(
        fixture.execute(&assemble(&source).unwrap()).unwrap().value,
        Value::Int32(1)
    );
}

#[test]
fn native_input_strings_reject_embedded_nul_and_unsupported_metadata() {
    let fixture = Fixture::new();
    for encoding in ["utf8", "utf16"] {
        let source = format!(
            ".module Test\n.entry Main\n.function Length(String) -> UIntPtr\n.foreign {} \"neoclr_utf8_len\" cdecl {encoding}\n.end\n.function Main() -> UIntPtr\nldstr \"a\\u0000b\"\ncall Length(String)\nret\n.end",
            fixture.import
        );
        let fault = fixture.execute(&assemble(&source).unwrap()).unwrap_err();
        assert!(fault.message.contains("embedded NUL"), "{fault:?}");
        assert_eq!(fault.function.as_deref(), Some("Main"));
    }
    for (parameters, returns, options) in [
        ("String", "Void", "cdecl"),
        ("", "String", "cdecl utf8"),
        ("String&", "Void", "cdecl utf16"),
        ("String", "Void", "cdecl ansi"),
        ("String", "Void", "cdecl utf8 utf16"),
    ] {
        assert!(assemble(&format!(".module Test\n.function F({parameters}) -> {returns}\n.foreign \"missing\" \"f\" {options}\n.end")).is_err());
    }
    let module = assemble(
        ".module Test\n.function F(String) -> Void\n.foreign \"missing\" \"f\" cdecl utf8\n.end",
    )
    .unwrap();
    let mut json = serde_json::to_value(&module).unwrap();
    json["functions"][0]["pinvoke"]["string_marshalling"] = "Ansi".into();
    assert!(load(&json.to_string()).is_err());
    json["functions"][0]["pinvoke"]["string_marshalling"] = serde_json::Value::Null;
    assert!(load(&json.to_string()).is_err());
}

#[test]
fn foreign_is_the_platform_neutral_import_directive_with_legacy_alias() {
    let legacy =
        ".module Test\n.function F(Int32) -> Int32\n.pinvoke \"missing\" \"f\" cdecl\n.end";
    let canonical = legacy.replace(".pinvoke", ".foreign");
    let canonical = assemble(&canonical).unwrap();
    assert_eq!(
        serde_json::to_value(assemble(legacy).unwrap()).unwrap(),
        serde_json::to_value(canonical).unwrap()
    );
    assert!(
        assemble(&legacy.replace(
            "cdecl\n.end",
            "cdecl\n.foreign \"missing\" \"f\" cdecl\n.end"
        ))
        .is_err()
    );
    let legacy_json = serde_json::to_value(assemble(legacy).unwrap()).unwrap();
    assert!(
        legacy_json["functions"][0]["pinvoke"]
            .get("string_marshalling")
            .is_none()
    );
}

#[test]
fn per_slot_marshalling_preserves_boolean_width_and_mixed_encodings() {
    let fixture = Fixture::new();
    let library: String = serde_json::from_str(&fixture.import).unwrap();
    for (kind, symbol) in [("Bool8", "neoclr_bool8"), ("Bool32", "neoclr_bool32")] {
        for (number, expected) in [(0, false), (1, true)] {
            let mut module = assemble(&format!(".module Test\n.entry Main\n.function F(Boolean) -> Boolean\nldarg 0\nret\n.end\n.function Main() -> Boolean\nldc.i4 {number}\nldc.i4 1\nceq\ncall F(Boolean)\nret\n.end")).unwrap();
            module.functions[0].body.clear();
            module.functions[0].pinvoke = Some(
                serde_json::from_value(serde_json::json!({
                    "library": library, "entry_point": symbol, "calling_convention": "Cdecl",
                    "parameter_marshalling": { "0": kind }, "return_marshalling": kind
                }))
                .unwrap(),
            );
            let module = load(&serde_json::to_string(&module).unwrap()).unwrap();
            assert_eq!(
                fixture.execute(&module).unwrap().value,
                Value::Boolean(expected)
            );
            for index in [0, 1] {
                let mut invalid = module.clone();
                invalid.functions[0]
                    .pinvoke
                    .as_mut()
                    .unwrap()
                    .parameter_marshalling
                    .insert(index.to_string(), neoclr::metadata::NativeMarshalling::Utf8);
                assert!(load(&serde_json::to_string(&invalid).unwrap()).is_err());
            }
        }
    }
    let mut module = assemble(".module Test\n.entry Main\n.function F(String,String) -> Int32\nldc.i4 0\nret\n.end\n.function Main() -> Int32\nldstr \"hé🦀\"\nldstr \"hé🦀\"\ncall F(String,String)\nret\n.end").unwrap();
    module.functions[0].body.clear();
    module.functions[0].pinvoke = Some(serde_json::from_value(serde_json::json!({
        "library": library, "entry_point": "neoclr_mixed_encoding", "calling_convention": "Cdecl",
        "string_marshalling": "Utf16", "parameter_marshalling": { "0": "Utf8" }
    })).unwrap());
    let module = load(&serde_json::to_string(&module).unwrap()).unwrap();
    assert_eq!(fixture.execute(&module).unwrap().value, Value::Int32(42));
    let system = fixture.module(
        "Int32,Int32",
        "Int32",
        "neoclr_system_add",
        "ldc.i4 19\nldc.i4 23",
    );
    let mut system = system;
    system.functions[0]
        .pinvoke
        .as_mut()
        .unwrap()
        .calling_convention = neoclr::metadata::CallingConvention::System;
    assert_eq!(fixture.execute(&system).unwrap().value, Value::Int32(42));
}

#[test]
fn returned_native_pointer_binds_to_structural_function() {
    let fixture = Fixture::new();
    let source = include_str!("../examples/foreign_function_pointer.neoil")
        .replace("\"./target/native/neoclr_sample\"", &fixture.import);
    let module = assemble(&source).unwrap();
    let module = load(&serde_json::to_string(&module).unwrap()).unwrap();
    let image = neoclr::metadata_container::write_module(&module).unwrap();
    let module = neoclr::metadata_container::load_envelope(&image).unwrap();
    assert_eq!(fixture.execute(&module).unwrap().value, Value::Int32(42));
}

#[test]
fn native_function_bindings_share_string_and_boolean_marshalling() {
    let fixture = Fixture::new();
    for (symbol, shape, parameters, abi, args, returns, expected) in [
        (
            "neoclr_get_utf8_pair",
            "fn<String,Int32,String,Int32>",
            "String,Int32,String",
            r#"{"calling_convention":"Cdecl","string_marshalling":"Utf16","parameter_marshalling":{"0":"Utf8","2":"Utf8"}}"#,
            "ldstr \"hé🦀\"\nldc.i4 41\nldstr \"世界\"",
            "Int32",
            Value::Int32(42),
        ),
        (
            "neoclr_get_bool8",
            "fn<Boolean,Boolean>",
            "Boolean",
            r#"{"calling_convention":"System","parameter_marshalling":{"0":"Bool8"},"return_marshalling":"Bool8"}"#,
            "ldc.bool true",
            "Boolean",
            Value::Boolean(true),
        ),
    ] {
        let source = format!(
            ".module Test\n.entry Main\n.function Factory() -> Void*\n.foreign {} {symbol:?} cdecl\n.end\n.function Main() -> {returns}\ncall Factory()\nfunction.bind.native {shape} = {abi}\n{args}\ncall instance {shape}::Invoke({parameters})\nret\n.end",
            fixture.import
        );
        assert_eq!(
            fixture.execute(&assemble(&source).unwrap()).unwrap().value,
            expected
        );
    }
}

#[test]
fn native_function_binding_rejects_null_data_and_untrusted_execution() {
    let fixture = Fixture::new();
    for (pointer, message) in [
        ("ptr.null Void", "null native function pointer"),
        ("ldc.i4 1\nheap.alloc Int32", "guest data storage"),
    ] {
        let source = format!(
            ".module Test\n.entry Main\n.function Main() -> fn<Int32>\n{pointer}\nfunction.bind.native fn<Int32> = {{\"calling_convention\":\"Cdecl\"}}\nret\n.end"
        );
        let module = assemble(&source).unwrap();
        assert!(
            run(&module, Limits::default())
                .unwrap_err()
                .message
                .contains("trusted")
        );
        let error = fixture.execute(&module).unwrap_err();
        assert!(error.message.contains(message), "{error:?}");
    }
    for (shape, abi) in [
        ("fn<Boolean>", r#"{"calling_convention":"Cdecl"}"#),
        (
            "fn<String>",
            r#"{"calling_convention":"Cdecl","return_marshalling":"Utf8"}"#,
        ),
        (
            "fn<Int32>",
            r#"{"calling_convention":"Cdecl","parameter_marshalling":{"0":"Bool8"}}"#,
        ),
        ("Int32", r#"{"calling_convention":"Cdecl"}"#),
    ] {
        assert!(assemble(&format!(".module Bad\n.function Main() -> {shape}\nptr.null Void\nfunction.bind.native {shape} = {abi}\nret\n.end")).is_err());
    }
}

#[test]
fn native_function_copies_share_identity_and_have_no_managed_target() {
    let fixture = Fixture::new();
    let prefix = format!(
        ".module Test\n.entry Main\n.function Factory() -> Void*\n.foreign {} \"neoclr_get_add\" cdecl\n.end\n",
        fixture.import
    );
    let binding = "call Factory()\nfunction.bind.native fn<Int32,Int32,Int32> = {\"calling_convention\":\"Cdecl\"}";
    let equal = assemble(&format!(
        "{prefix}.function Main() -> Boolean\n{binding}\ndup\nceq\nret\n.end"
    ))
    .unwrap();
    assert_eq!(fixture.execute(&equal).unwrap().value, Value::Boolean(true));
    let distinct = assemble(&format!(
        "{prefix}.function Main() -> Boolean\n{binding}\n{binding}\nceq\nret\n.end"
    ))
    .unwrap();
    assert_eq!(
        fixture.execute(&distinct).unwrap().value,
        Value::Boolean(false)
    );
    let name = assemble(&format!("{prefix}.function Main() -> String\n{binding}\ncall instance fn<Int32,Int32,Int32>::ToString()\nret\n.end")).unwrap();
    assert_eq!(
        fixture.execute(&name).unwrap().value,
        Value::String("native Function".into())
    );
    let target = assemble(&format!("{prefix}.function Main() -> Int32\n{binding}\ncall instance fn<Int32,Int32,Int32>::get_Function()\npop\nldc.i4 0\nret\n.end")).unwrap();
    assert!(
        fixture
            .execute(&target)
            .unwrap_err()
            .message
            .contains("no managed method target")
    );
}
