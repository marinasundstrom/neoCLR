//! Exercise the experimental resource protocol below any frontend/library checks.
use neoclr::{assemble, assembler::parse_function_ref, Limits, LoadedProgram, Value};
const SERVICES: &str = include_str!("../runtime/neoCLR/Runtime/FileStreams.neoil");
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
    invoke_with_limits(body, returns, path, Limits::default())
}
fn invoke_with_limits(
    body: &str,
    returns: &str,
    path: &str,
    limits: Limits,
) -> Result<neoclr::Execution, neoclr::Fault> {
    let module = program(body, returns);
    let loaded = LoadedProgram::new(&module).unwrap();
    loaded.verify().unwrap();
    loaded
        .resolve_function(&parse_function_ref("Run(String)").unwrap())
        .unwrap()
        .invoke(vec![Value::String(path.into())], limits)
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

const READ_BUFFER: &str = r#"
.local Int32 handle
.local arrayref<Byte> bytes
.local arrayref<Byte> alias
ldarg path
call neoCLR.Runtime.FileOpenRead(String)
value.unpack Int32
stloc handle
ldc.i4 5
newarr Byte
stloc bytes
ldloc bytes
stloc alias
ldloc bytes
ldc.i4 4
ldc.i4 99
conv.u1
stelem Byte
"#;

fn read_into(offset: i32, count: i32, outcome: &str, expected: i32) -> String {
    format!(
        r#"
ldloc handle
ldloc bytes
ldc.i4 {offset}
ldc.i4 {count}
call neoCLR.Runtime.FileReadInto(Int32,arrayref<Byte>,Int32,Int32)
value.unpack {outcome}
{conversion}
ldc.i4 {expected}
beq Matched{label}
fault "Unexpected read outcome"
Matched{label}:
"#,
        conversion = if outcome == "Byte" { "conv.i4" } else { "" },
        label = format!("{offset}_{count}_{outcome}").replace('-', "N")
    )
}

#[test]
fn read_into_preserves_aliases_prefix_tail_and_eof_buffer() {
    let fixture = Fixture::new();
    std::fs::write(fixture.path(), b"ABC").unwrap();
    let mut body = READ_BUFFER.to_owned();
    // Zero-size at the end is valid and does not consume input.
    body += &read_into(5, 0, "Int32", 0);
    body += &read_into(1, 2, "Int32", 2);
    // Only C is left: a short read must leave the rest of the buffer alone.
    body += &read_into(3, 2, "Int32", 1);
    body += &read_into(0, 5, "Int32", 0);
    for (index, expected) in [0, 65, 66, 67, 99].iter().enumerate() {
        body += &format!(
            r#"
ldloc alias
ldc.i4 {index}
ldelem Byte
conv.i4
ldc.i4 {expected}
beq Element{index}
fault "Read changed wrong buffer element"
Element{index}:
"#
        );
    }
    body += "ldloc handle\ncall neoCLR.Runtime.FileClose(Int32)\npop\n";
    body += &read_into(0, 1, "Byte", 6);
    body += "ldc.i4 1";
    assert_eq!(
        invoke(&body, "Int32", &fixture.path()).unwrap().value,
        Value::Int32(1)
    );
}

#[test]
fn invalid_read_ranges_do_not_mutate_buffer_or_advance_file() {
    for (offset, count) in [(-1, 1), (0, -1), (4, 2), (6, 0), (i32::MAX, i32::MAX)] {
        let fixture = Fixture::new();
        std::fs::write(fixture.path(), b"AB").unwrap();
        let mut body = format!("{READ_BUFFER}{}", read_into(offset, count, "Byte", 7));
        for (index, expected) in [0, 0, 0, 0, 99].iter().enumerate() {
            body += &format!(
                "ldloc bytes\nldc.i4 {index}\nldelem Byte\nconv.i4\nldc.i4 {expected}\nbeq Preserved{index}\nfault \"Invalid read changed buffer\"\nPreserved{index}:\n"
            );
        }
        body += &read_into(0, 1, "Int32", 1);
        body += "ldloc bytes\nldc.i4 0\nldelem Byte";
        assert_eq!(
            invoke(&body, "Byte", &fixture.path()).unwrap().value,
            Value::Byte(b'A')
        );
    }
}

#[test]
fn oversized_read_into_is_rejected_before_advancing() {
    let fixture = Fixture::new();
    std::fs::write(fixture.path(), b"A").unwrap();
    let body = format!(
        "{}{}{}ldloc bytes\nldc.i4 0\nldelem Byte",
        READ_BUFFER.replace("ldc.i4 5\nnewarr", "ldc.i4 65537\nnewarr"),
        read_into(0, 65537, "Byte", 8),
        read_into(0, 1, "Int32", 1)
    );
    assert_eq!(
        invoke_with_limits(
            &body,
            "Byte",
            &fixture.path(),
            Limits {
                array_elements: 131_072,
                array_bytes: 131_072 * std::mem::size_of::<Value>(),
                ..Limits::default()
            }
        )
        .unwrap()
        .value,
        Value::Byte(b'A')
    );
}

#[test]
fn read_into_requires_read_access_and_exact_byte_array_signature() {
    let fixture = Fixture::new();
    std::fs::write(fixture.path(), b"unchanged").unwrap();
    let body = format!(
        "{}{}ldc.i4 1",
        READ_BUFFER.replace("FileOpenRead", "FileOpenWrite"),
        read_into(0, 1, "Byte", 9)
    );
    invoke(&body, "Int32", &fixture.path()).unwrap();
    assert_eq!(std::fs::read(fixture.path()).unwrap(), b"unchanged");
    let invalid = ".module Bad\n.function neoCLR.Runtime.FileReadInto(Int32 handle,arrayref<Int32> bytes,Int32 offset,Int32 count) -> Value\n.methodimpl InternalCall\n.end";
    assert!(assemble(invalid).is_err());
}

#[cfg(unix)]
#[test]
fn metadata_follows_links_and_reports_dangling_targets() {
    let fixture = Fixture::new();
    let target = fixture.path();
    let link = fixture.0.join("link");
    std::fs::write(&target, b"data").unwrap();
    std::os::unix::fs::symlink(&target, &link).unwrap();
    let path = link.to_str().unwrap();
    assert_eq!(
        invoke(
            "ldarg path\ncall neoCLR.Runtime.StorageKind(String)\nvalue.unpack Int32",
            "Int32",
            path
        )
        .unwrap()
        .value,
        Value::Int32(1)
    );
    std::fs::remove_file(&target).unwrap();
    assert_eq!(
        invoke(
            "ldarg path\ncall neoCLR.Runtime.StorageKind(String)\nvalue.unpack Byte",
            "Byte",
            path
        )
        .unwrap()
        .value,
        Value::Byte(2)
    );
}
