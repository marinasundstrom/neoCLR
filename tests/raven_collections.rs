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
fn profile_module(source: &str) -> Module {
    neoclr::assembler::read_modules(&[neoclr::assembler::ModuleInput::Source(source)], library())
        .unwrap()
        .remove(0)
}
fn program(body: &str, extra: &str) -> LoadedProgram {
    let app = profile_module(&format!(
        ".module Test\n.entry Main\n{extra}\n.function Main() -> Int32\n{body}\nret\n.end"
    ));
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
    body += "ldloc list\ncall instance System.Collections.ArrayList<Int32>::get_Capacity()\nldc.i4 16\nbeq Good\nfault \"growth\"\nGood:\nldloc alias\nldc.i4 8\ncallvirt instance System.Collections.MutableSequence<Int32>::get_Item(Int32)";
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

const MAP_CALLBACKS: &str = r#"
.function Equal(Int32 a,Int32 b) -> Boolean
ldarg a
ldarg b
ceq
ret
.end
.function Hash(Int32 key) -> Int32
ldc.i4 -2147483648
ret
.end
"#;
const MAP_START: &str = r#"
.local System.Collections.HashMap<Int32,Int32> map
.local System.Collections.Map<Int32,Int32> reader
.local System.Collections.MutableMap<Int32,Int32> writer
delegate.bind System.Func<Int32,Int32,Boolean> = Equal(Int32,Int32)
delegate.bind System.Func<Int32,Int32> = Hash(Int32)
newobj instance System.Collections.HashMap<Int32,Int32>::.ctor(System.Func<Int32,Int32,Boolean>,System.Func<Int32,Int32>)
stloc map
ldloc map
stloc reader
ldloc map
stloc writer
"#;

#[test]
fn map_collision_growth_duplicate_and_missing_outcomes_survive_gc() {
    let mut body = MAP_START.to_string();
    // A deliberately constant minimum integer hash exercises collision chains and
    // signed-hash normalization through several resizes.
    for i in 0..40 {
        body += &format!(
            "ldloc writer\nldc.i4 {i}\nldc.i4 {}\ncallvirt instance System.Collections.MutableMap<Int32,Int32>::TryAdd(Int32,Int32)\nbrtrue Added{i}\nfault \"insertion rejected\"\nAdded{i}:\n",
            i * 3
        );
    }
    body += "ldloc writer\nldc.i4 0\nldc.i4 999\ncallvirt instance System.Collections.MutableMap<Int32,Int32>::TryAdd(Int32,Int32)\nbrfalse Duplicate\nfault \"duplicate inserted\"\nDuplicate:\n";
    for i in 0..40 {
        body += &format!(
            "ldloc reader\nldc.i4 {i}\ncallvirt instance System.Collections.Map<Int32,Int32>::Find(Int32)\ncall instance System.Option<Int32>::GetSomeCase()\ncall instance System.Option.Some<Int32>::get_Value()\nldc.i4 {}\nbeq Found{i}\nfault \"wrong value\"\nFound{i}:\n",
            i * 3
        );
    }
    body += "ldloc reader\nldc.i4 100\ncallvirt instance System.Collections.Map<Int32,Int32>::Find(Int32)\ncall instance System.Option<Int32>::get_IsNone()\nbrtrue Absent\nfault \"absence lost\"\nAbsent:\nldloc reader\ncallvirt instance System.Collections.Map<Int32,Int32>::get_Count()";
    for callbacks in [
        MAP_CALLBACKS.to_string(),
        MAP_CALLBACKS.replace("ldc.i4 -2147483648", "ldarg key"),
    ] {
        let result = program(&body, &callbacks)
            .run(Limits {
                heap_objects: 24,
                instructions: 1_000_000,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(result.value, Value::Int32(40));
        assert!(result.heap.statistics().collections > 0);
        assert_eq!(result.memory.live_allocations(), 0);
    }
}

#[test]
fn map_key_snapshot_does_not_expose_storage_and_set_preserves_existing_key() {
    let body = format!(
        r#"
.local System.Collections.Sequence<Int32> keys
.local System.Collections.MutableSequence<Int32> writable
{MAP_START}
ldloc writer
ldc.i4 1
ldc.i4 7
callvirt instance System.Collections.MutableMap<Int32,Int32>::Set(Int32,Int32)
ldloc reader
callvirt instance System.Collections.Map<Int32,Int32>::get_Keys()
stloc keys
ldloc keys
castclass System.Collections.MutableSequence<Int32>
stloc writable
ldloc writable
ldc.i4 0
ldc.i4 999
callvirt instance System.Collections.MutableSequence<Int32>::set_Item(Int32,Int32)
ldloc writer
ldc.i4 1
ldc.i4 42
callvirt instance System.Collections.MutableMap<Int32,Int32>::Set(Int32,Int32)
ldloc writer
ldc.i4 2
ldc.i4 8
callvirt instance System.Collections.MutableMap<Int32,Int32>::Set(Int32,Int32)
ldloc keys
callvirt instance System.Collections.Collection<Int32>::get_Count()
ldc.i4 1
beq Snapshot
fault "snapshot changed size"
Snapshot:
ldloc reader
ldc.i4 999
callvirt instance System.Collections.Map<Int32,Int32>::ContainsKey(Int32)
brfalse Isolated
fault "snapshot changed map key"
Isolated:
ldloc reader
ldc.i4 1
callvirt instance System.Collections.Map<Int32,Int32>::Find(Int32)
call instance System.Option<Int32>::GetSomeCase()
call instance System.Option.Some<Int32>::get_Value()
"#
    );
    assert_eq!(
        program(&body, MAP_CALLBACKS)
            .run(Limits::default())
            .unwrap()
            .value,
        Value::Int32(42)
    );
}

#[test]
fn map_read_contract_cannot_mutate_and_type_arguments_remain_invariant() {
    let text = format!(
        ".module Test\n.entry Main\n{MAP_CALLBACKS}\n.function Main() -> Int32\n{MAP_START}\nldloc reader\nldc.i4 1\nldc.i4 2\ncallvirt instance System.Collections.Map<Int32,Int32>::Set(Int32,Int32)\nldc.i4 0\nret\n.end"
    );
    assert!(
        neoclr::assembler::read_modules(
            &[neoclr::assembler::ModuleInput::Source(&text)],
            library()
        )
        .is_err()
    );
    let m = profile_module(&format!(
        ".module Test\n.entry Main\n{MAP_CALLBACKS}\n.function Main() -> Int32\n{MAP_START}\nldloc reader\ncastclass System.Collections.Map<Int32,String>\npop\nldc.i4 0\nret\n.end"
    ));
    let p = LoadedProgram::with_library(&m, library()).unwrap();
    assert!(p.verify().is_err() || p.run(Limits::default()).is_err());
}

#[test]
fn map_retains_reference_keys_and_values_across_collection() {
    let extra = r#"
.type class Box
.field Value Int32
.end
.function Equal(Box a,Box b) -> Boolean
ldarg a
ldfld Box::Value
ldarg b
ldfld Box::Value
ceq
ret
.end
.function Hash(Box key) -> Int32
ldarg key
ldfld Box::Value
ret
.end
"#;
    let mut body = String::from(
        r#"
.local System.Collections.HashMap<Box,Box> map
.local Box key
.local Box value
delegate.bind System.Func<Box,Box,Boolean> = Equal(Box,Box)
delegate.bind System.Func<Box,Int32> = Hash(Box)
newobj instance System.Collections.HashMap<Box,Box>::.ctor(System.Func<Box,Box,Boolean>,System.Func<Box,Int32>)
stloc map
ldc.i4 7
newobj Box
stloc key
ldc.i4 1
newobj Box
stloc value
ldloc map
ldloc key
ldloc value
call instance System.Collections.HashMap<Box,Box>::Set(Box,Box)
ldloc map
ldc.i4 7
newobj Box
ldloc value
call instance System.Collections.HashMap<Box,Box>::Set(Box,Box)
ldloc map
call instance System.Collections.HashMap<Box,Box>::get_Keys()
ldc.i4 0
callvirt instance System.Collections.Sequence<Box>::get_Item(Int32)
ldloc key
ref.eq
brtrue OriginalKey
fault "replacement changed key identity"
OriginalKey:
ldloc value
ldc.i4 42
stfld Box::Value
ldloca key
initobj Box
ldloca value
initobj Box
"#,
    );
    for i in 0..60 {
        body += &format!("ldc.i4 {i}\nnewobj Box\npop\n");
    }
    body += "ldloc map\nldc.i4 7\nnewobj Box\ncall instance System.Collections.HashMap<Box,Box>::Find(Box)\ncall instance System.Option<Box>::GetSomeCase()\ncall instance System.Option.Some<Box>::get_Value()\nldfld Box::Value";
    let result = program(&body, extra)
        .run(Limits {
            heap_objects: 24,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert!(result.heap.statistics().collections > 0);
}

#[test]
fn map_reentrant_callback_faults_instead_of_corrupting_chains() {
    let extra = format!(
        r#"
{MAP_CALLBACKS}
.type class CallbackState
.field Map System.Collections.HashMap<Int32,Int32>
.method instance Reenter(Int32 key) -> Int32
ldarg this
ldfld CallbackState::Map
ldarg key
call instance System.Collections.HashMap<Int32,Int32>::Find(Int32)
pop
ldc.i4 0
ret
.end
.end
"#
    );
    let body = r#"
.local CallbackState state
.local System.Collections.HashMap<Int32,Int32> map
ldloca map
initobj System.Collections.HashMap<Int32,Int32>
ldloc map
newobj CallbackState
stloc state
delegate.bind System.Func<Int32,Int32,Boolean> = Equal(Int32,Int32)
ldloc state
delegate.bind System.Func<Int32,Int32> = instance CallbackState::Reenter(Int32)
newobj instance System.Collections.HashMap<Int32,Int32>::.ctor(System.Func<Int32,Int32,Boolean>,System.Func<Int32,Int32>)
stloc map
ldloc state
ldloc map
stfld CallbackState::Map
ldloc map
ldc.i4 1
ldc.i4 2
call instance System.Collections.HashMap<Int32,Int32>::TryAdd(Int32,Int32)
pop
ldc.i4 0
"#;
    let error = program(body, &extra).run(Limits::default()).unwrap_err();
    assert!(
        error.message.contains("callbacks must not reenter"),
        "{error:?}"
    );
}

#[test]
fn raven_array_iterator_observes_shared_elements_and_disposal_is_terminal() {
    let body = r#"
.local arrayref<Int32> data
.local System.Collections.Iterator<Int32> iterator
.local Int32 observed
ldc.i4 1
newarr Int32
stloc data
ldloc data
castclass System.Collections.Iterable<Int32>
callvirt instance System.Collections.Iterable<Int32>::GetIterator()
stloc iterator
ldloc data
ldc.i4 0
ldc.i4 42
stelem Int32
ldloc iterator
callvirt instance System.Collections.Iterator<Int32>::MoveNext()
brtrue Read
fault "missing element"
Read:
ldloc iterator
callvirt instance System.Collections.Iterator<Int32>::get_Current()
stloc observed
ldloc iterator
callvirt instance System.Disposable::Dispose()
ldloc iterator
callvirt instance System.Collections.Iterator<Int32>::MoveNext()
brfalse Done
fault "disposed iterator resumed"
Done:
ldloc observed
"#;
    assert_eq!(
        program(body, "").run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn raven_array_iterator_current_requires_a_live_position() {
    for transition in [
        "",
        "ldloc iterator\ncallvirt instance System.Disposable::Dispose()\n",
        "ldloc iterator\ncallvirt instance System.Collections.Iterator<Int32>::MoveNext()\npop\n",
    ] {
        let body = format!(
            r#"
.local System.Collections.Iterator<Int32> iterator
ldc.i4 0
newarr Int32
castclass System.Collections.Iterable<Int32>
callvirt instance System.Collections.Iterable<Int32>::GetIterator()
stloc iterator
{transition}
ldloc iterator
callvirt instance System.Collections.Iterator<Int32>::get_Current()
"#
        );
        let error = program(&body, "").run(Limits::default()).unwrap_err();
        assert!(
            error.message.contains("Iterator has no current element"),
            "{error:?}"
        );
    }
}
