//! Exercise the experimental resource protocol below any frontend/library checks.
use neoclr::{Limits, LoadedProgram, Value, assemble, assembler::parse_function_ref};
const SERVICES: &str = include_str!("../docs/experiments/file-streams/Services.neoil");
struct Fixture(std::path::PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "neoclr-stream-vm-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn path(&self) -> String {
        self.0.join("data").to_str().unwrap().into()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
fn program(body: &str, returns: &str) -> neoclr::Module {
    assemble(&format!(".module FileResources\n{SERVICES}\n.function Run(String path) -> {returns}\n{body}\nret\n.end")).unwrap()
}
fn invoke(body: &str, returns: &str, path: &str) -> Result<neoclr::Execution, neoclr::Fault> {
    let module = program(body, returns);
    let loaded = LoadedProgram::new(&module).unwrap();
    loaded.verify().unwrap();
    loaded
        .resolve_function(&parse_function_ref("Run(String)").unwrap())
        .unwrap()
        .invoke(vec![Value::String(path.into())], Limits::default())
}
#[test]
fn disk_round_trip_through_vm_resource_services() {
    let fixture = Fixture::new();
    let result = invoke(
        r#"
.local Int32 handle
.local arrayref<Byte> bytes
ldarg path
call neoCLR.Runtime.FileCreateNew(String)
value.unpack Int32
stloc handle
ldc.i4 3
newarr Byte
stloc bytes
ldloc bytes
ldc.i4 0
ldc.i4 65
conv.u1
stelem Byte
ldloc bytes
ldc.i4 1
ldc.i4 66
conv.u1
stelem Byte
ldloc bytes
ldc.i4 2
ldc.i4 67
conv.u1
stelem Byte
ldloc handle
ldloc bytes
ldc.i4 0
ldc.i4 3
call neoCLR.Runtime.FileWriteChunk(Int32,arrayref<Byte>,Int32,Int32)
value.unpack Int32
ldc.i4 3
beq Written
fault "Unexpected write count"
Written:
ldloc handle
call neoCLR.Runtime.FileFlush(Int32)
pop
ldloc handle
call neoCLR.Runtime.FileClose(Int32)
pop
ldarg path
call neoCLR.Runtime.FileOpenRead(String)
value.unpack Int32
stloc handle
ldloc handle
ldc.i4 64
call neoCLR.Runtime.FileReadChunk(Int32,Int32)
value.unpack Byte[]
"#,
        "Byte[]",
        &fixture.path(),
    )
    .unwrap();
    assert_eq!(
        result.value,
        Value::Array {
            element: neoclr::assembler::parse_type("Byte").unwrap(),
            elements: b"ABC".iter().copied().map(Value::Byte).collect(),
        }
    );
    assert_eq!(std::fs::read(fixture.path()).unwrap(), b"ABC");
    // Reader was deliberately not closed by guest code. Invocation teardown owns it.
    std::fs::remove_file(fixture.path()).unwrap();
}
#[test]
fn closed_handle_cannot_access_later_file() {
    let fixture = Fixture::new();
    std::fs::write(fixture.path(), b"x").unwrap();
    let result = invoke(
        r#"
.local Int32 old
ldarg path
call neoCLR.Runtime.FileOpenRead(String)
value.unpack Int32
stloc old
ldloc old
call neoCLR.Runtime.FileClose(Int32)
pop
ldarg path
call neoCLR.Runtime.FileOpenRead(String)
pop
ldloc old
ldc.i4 1
call neoCLR.Runtime.FileReadChunk(Int32,Int32)
value.unpack Byte
"#,
        "Byte",
        &fixture.path(),
    )
    .unwrap();
    assert_eq!(result.value, Value::Byte(6));
}
#[test]
fn invocations_do_not_share_handle_tables_and_faults_release_files() {
    let fixture = Fixture::new();
    let fault = invoke(
        "ldarg path\ncall neoCLR.Runtime.FileCreateNew(String)\npop\nfault \"after open\"",
        "Int32",
        &fixture.path(),
    )
    .unwrap_err();
    assert!(fault.message.contains("after open"));
    let result = invoke(
        "ldc.i4 1\nldc.i4 1\ncall neoCLR.Runtime.FileReadChunk(Int32,Int32)\nvalue.unpack Byte",
        "Byte",
        &fixture.path(),
    )
    .unwrap();
    assert_eq!(result.value, Value::Byte(6));
    std::fs::remove_file(fixture.path()).unwrap();
}
#[test]
fn storage_kind_and_directory_creation_keep_errors_distinct() {
    let fixture = Fixture::new();
    let path = fixture.path();
    assert_eq!(
        invoke(
            "ldarg path\ncall neoCLR.Runtime.StorageKind(String)\nvalue.unpack Byte",
            "Byte",
            &path
        )
        .unwrap()
        .value,
        Value::Byte(2)
    );
    assert_eq!(
        invoke(
            "ldarg path\ncall neoCLR.Runtime.StorageCreateDirectory(String)\nvalue.unpack Int32",
            "Int32",
            &path
        )
        .unwrap()
        .value,
        Value::Int32(0)
    );
    assert_eq!(
        invoke(
            "ldarg path\ncall neoCLR.Runtime.StorageKind(String)\nvalue.unpack Int32",
            "Int32",
            &path
        )
        .unwrap()
        .value,
        Value::Int32(2)
    );
    assert_eq!(
        invoke(
            "ldarg path\ncall neoCLR.Runtime.StorageCreateDirectory(String)\nvalue.unpack Byte",
            "Byte",
            &path
        )
        .unwrap()
        .value,
        Value::Byte(5)
    );
}
#[test]
fn host_dispatch_requires_exact_signature() {
    let source = ".module Bad\n.function neoCLR.Runtime.FileOpenRead(Int32 path) -> Value\n.methodimpl InternalCall\n.end";
    assert!(assemble(source).is_err());
}

#[test]
fn invalid_write_slice_does_not_change_the_file() {
    let fixture = Fixture::new();
    std::fs::write(fixture.path(), b"original").unwrap();
    let result = invoke(
        r#"
.local Int32 handle
.local arrayref<Byte> bytes
ldarg path
call neoCLR.Runtime.FileOpenWrite(String)
value.unpack Int32
stloc handle
ldc.i4 2
newarr Byte
stloc bytes
ldloc handle
ldloc bytes
ldc.i4 -1
ldc.i4 2
call neoCLR.Runtime.FileWriteChunk(Int32,arrayref<Byte>,Int32,Int32)
value.unpack Byte
"#,
        "Byte",
        &fixture.path(),
    )
    .unwrap();
    assert_eq!(result.value, Value::Byte(7));
    assert_eq!(std::fs::read(fixture.path()).unwrap(), b"original");
}

#[test]
fn a_managed_function_named_like_a_service_does_not_open_a_file() {
    let fixture = Fixture::new();
    let module = assemble(".module Managed\n.function neoCLR.Runtime.FileCreateNew(String path) -> Value\nldc.i4 42\nvalue.pack Int32\nret\n.end").unwrap();
    let loaded = LoadedProgram::new(&module).unwrap();
    let result = loaded
        .resolve_function(&parse_function_ref("neoCLR.Runtime.FileCreateNew(String)").unwrap())
        .unwrap()
        .invoke(vec![Value::String(fixture.path())], Limits::default())
        .unwrap();
    assert_eq!(result.value, Value::Erased(Box::new(Value::Int32(42))));
    assert!(!std::path::Path::new(&fixture.path()).exists());
}
