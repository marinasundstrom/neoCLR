//! The isolated target profile executes the existing collection algorithms with
//! ordinary class/interface/array references. Neo's bundled profile is unchanged.
use neoclr::{Limits, LoadedProgram, Module, Value, assemble};
use std::{process::Command, sync::OnceLock};

fn library() -> &'static Module {
    static LIBRARY: OnceLock<Module> = OnceLock::new();
    LIBRARY.get_or_init(|| {
        let output = Command::new("python3")
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .args(["-c", "import runpy; m=runpy.run_path('docs/experiments/raven-target/collection_library.py'); print(m['build'](m['ROOT'] / 'runtime/System.neoil'))"])
            .output().unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assemble(std::str::from_utf8(&output.stdout).unwrap()).unwrap()
    })
}
fn program(body: &str, extra: &str) -> LoadedProgram {
    let app = assemble(&format!(
        ".module Test\n.entry Main\n{extra}\n.function Main() -> Int32\n{body}\nret\n.end"
    ))
    .unwrap();
    let program = LoadedProgram::with_library(&app, library()).unwrap();
    program.verify().unwrap();
    program
}
const START: &str = ".local System.Collections.ArrayList<Int32> list\nldc.i4 0\nnewobj instance System.Collections.ArrayList<Int32>::.ctor(Int32)\nstloc list\n";

#[test]
fn aliases_grow_and_dispatch_through_inherited_interfaces() {
    let mut body =
        format!(".local System.Collections.List<Int32> alias\n{START}ldloc list\nstloc alias\n");
    for i in 0..9 {
        body += &format!(
            "ldloc alias\nldc.i4 {i}\ncallvirt instance System.Collections.List<Int32>::Add(Int32)\n"
        );
    }
    body += "ldloc list\ncall instance System.Collections.ArrayList<Int32>::get_Capacity()\nldc.i4 16\nbeq Good\nfault \"growth\"\nGood:\nldloc alias\nldc.i4 8\ncallvirt instance System.Collections.List<Int32>::get_Item(Int32)";
    let result = program(&body, "")
        .run(Limits {
            heap_objects: 5,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(result.value, Value::Int32(8));
    assert!(result.heap.statistics().collections > 0);
    assert_eq!(result.memory.live_allocations(), 0);
}

#[test]
fn copy_is_independent_while_assignment_shares_identity() {
    let body = format!(
        r#".local System.Collections.ArrayList<Int32> copy
{START}
ldloc list
ldc.i4 42
call instance System.Collections.ArrayList<Int32>::Add(Int32)
ldloc list
call instance System.Collections.ArrayList<Int32>::Copy()
stloc copy
ldloc list
ldloc copy
ref.eq
brfalse Distinct
fault "Copy shared identity"
Distinct:
ldloc list
ldc.i4 0
ldc.i4 99
call instance System.Collections.ArrayList<Int32>::set_Item(Int32,Int32)
ldloc copy
ldc.i4 0
call instance System.Collections.ArrayList<Int32>::get_Item(Int32)
"#
    );
    assert_eq!(
        program(&body, "").run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn returned_iterator_retains_buffer_through_gc_and_disposes_idempotently() {
    let extra = r#"
.function Make() -> System.Collections.Iterator<Int32>
.local System.Collections.ArrayList<Int32> list
ldc.i4 1
newobj instance System.Collections.ArrayList<Int32>::.ctor(Int32)
stloc list
ldloc list
ldc.i4 42
call instance System.Collections.ArrayList<Int32>::Add(Int32)
ldloc list
call instance System.Collections.ArrayList<Int32>::GetIterator()
ret
.end
"#;
    let body = r#"
.local System.Collections.Iterator<Int32> iterator
.local Int32 result
call Make()
stloc iterator
ldc.i4 1
newarr Int32
pop
ldc.i4 1
newarr Int32
pop
ldloc iterator
callvirt instance System.Collections.Iterator<Int32>::MoveNext()
brtrue Current
fault "empty"
Current:
ldloc iterator
callvirt instance System.Collections.Iterator<Int32>::get_Current()
stloc result
ldloc iterator
callvirt instance System.Disposable::Dispose()
ldloc iterator
callvirt instance System.Disposable::Dispose()
ldloc iterator
callvirt instance System.Collections.Iterator<Int32>::MoveNext()
brfalse End
fault "disposed iterator advanced"
End:
ldloc result
"#;
    let result = program(body, extra)
        .run(Limits {
            heap_objects: 5,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert!(result.heap.statistics().collections > 0);
}

#[test]
fn iterator_captures_original_buffer_and_extent() {
    let body = r#"
.local System.Collections.ArrayList<Int32> list
.local System.Collections.Iterator<Int32> iterator
ldc.i4 1
newobj instance System.Collections.ArrayList<Int32>::.ctor(Int32)
stloc list
ldloc list
ldc.i4 42
call instance System.Collections.ArrayList<Int32>::Add(Int32)
ldloc list
call instance System.Collections.ArrayList<Int32>::GetIterator()
stloc iterator
ldloc list
ldc.i4 99
call instance System.Collections.ArrayList<Int32>::Add(Int32)
ldloc list
ldc.i4 0
ldc.i4 7
call instance System.Collections.ArrayList<Int32>::set_Item(Int32,Int32)
ldloc iterator
callvirt instance System.Collections.Iterator<Int32>::MoveNext()
pop
ldloc iterator
callvirt instance System.Collections.Iterator<Int32>::get_Current()
ldloc iterator
callvirt instance System.Collections.Iterator<Int32>::MoveNext()
brfalse End
fault "extent changed"
End:
"#;
    assert_eq!(
        program(body, "").run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn invalid_capacity_index_and_current_fault() {
    for (body, message) in [
        (
            "ldc.i4 -1\nnewobj instance System.Collections.ArrayList<Int32>::.ctor(Int32)\npop\nldc.i4 0"
                .to_string(),
            "capacity must be non-negative",
        ),
        (
            format!(
                "{START}ldloc list\nldc.i4 0\ncall instance System.Collections.ArrayList<Int32>::get_Item(Int32)"
            ),
            "index out of range",
        ),
        (
            format!(
                "{START}ldloc list\ncall instance System.Collections.ArrayList<Int32>::GetIterator()\ncallvirt instance System.Collections.Iterator<Int32>::get_Current()"
            ),
            "no current element",
        ),
    ] {
        assert!(
            program(&body, "")
                .run(Limits::default())
                .unwrap_err()
                .message
                .contains(message)
        );
    }
}

#[test]
fn wrong_element_type_is_rejected() {
    let app = assemble(&format!(".module Wrong\n.entry Main\n.function Main() -> noresult\n{START}ldloc list\nldstr \"wrong\"\ncall instance System.Collections.ArrayList<Int32>::Add(Int32)\nret\n.end")).unwrap();
    let p = LoadedProgram::with_library(&app, library()).unwrap();
    assert!(p.verify().is_err());
    assert!(p.run(Limits::default()).is_err());
}

#[test]
fn class_elements_keep_identity_and_generic_void_remains_valid() {
    let extra = ".type class Cell\n.field Value Int32\n.end";
    let body = r#"
.local Cell cell
.local System.Collections.ArrayList<Cell> list
ldc.i4 42
newobj Cell
stloc cell
ldc.i4 1
newobj instance System.Collections.ArrayList<Cell>::.ctor(Int32)
stloc list
ldloc list
ldloc cell
call instance System.Collections.ArrayList<Cell>::Add(Cell)
ldloc cell
ldloc list
ldc.i4 0
call instance System.Collections.ArrayList<Cell>::get_Item(Int32)
ref.eq
brtrue Shared
fault "element was copied"
Shared:
ldc.i4 0
newobj instance System.Collections.ArrayList<Void>::.ctor(Int32)
ldvoid
call instance System.Collections.ArrayList<Void>::Add(Void)
ldc.i4 42
"#;
    assert_eq!(
        program(body, extra).run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn constructors_start_empty_with_requested_capacity_and_grow() {
    for (argument, signature, capacity) in [
        ("", "", 0),
        ("ldc.i4 0\n", "Int32", 0),
        ("ldc.i4 3\n", "Int32", 3),
    ] {
        let body = format!(
            ".local System.Collections.ArrayList<Int32> list\n{argument}newobj instance System.Collections.ArrayList<Int32>::.ctor({signature})\nstloc list\nldloc list\ncall instance System.Collections.ArrayList<Int32>::get_Count()\nldc.i4 0\nbeq Empty\nfault \"count\"\nEmpty:\nldloc list\ncall instance System.Collections.ArrayList<Int32>::get_Capacity()\nldc.i4 {capacity}\nbeq Capacity\nfault \"capacity\"\nCapacity:\nldloc list\nldc.i4 42\ncall instance System.Collections.ArrayList<Int32>::Add(Int32)\nldloc list\nldc.i4 0\ncall instance System.Collections.ArrayList<Int32>::get_Item(Int32)"
        );
        assert_eq!(
            program(&body, "").run(Limits::default()).unwrap().value,
            Value::Int32(42)
        );
    }
}
