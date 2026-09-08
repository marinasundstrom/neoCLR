use neoclr::{Limits, LoadedProgram, Value, assemble, load};

fn program(body: &str, returns: &str, extra: &str) -> LoadedProgram {
    let module = assemble(&format!(
        ".module App\n.entry Main\n{extra}\n.function Main() -> {returns}\n{body}\nret\n.end"
    ))
    .unwrap();
    let program =
        LoadedProgram::new(&load(&serde_json::to_string(&module).unwrap()).unwrap()).unwrap();
    program.verify().unwrap();
    program
}

#[test]
fn sample_grows_through_reference_alias_without_native_allocations() {
    let module = assemble(include_str!("../examples/array_list.neoil")).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    let result = program.run(Limits::default()).unwrap();
    assert_eq!(
        result.output,
        ["ArrayList count:", "5", "0", "1", "4", "9", "16"]
    );
    assert_eq!(result.memory.live_allocations(), 0);
    assert!(result.heap.statistics().allocated_objects >= 3);
    assert_eq!(result.heap.len(), 0);
}

#[test]
fn growth_copies_live_elements_and_keeps_amortized_capacity_policy() {
    for initial in [0, 1, 4] {
        let mut body = format!(
            ".local System.Collections.ArrayList<Int32> list\nldc.i4 {initial}\ncall System.Collections.ArrayList<Int32>::Allocate(Int32)\nstloc list\n"
        );
        for i in 0..9 {
            body += &format!(
                "ldloca list\nldc.i4 {i}\ncall instance System.Collections.ArrayList<Int32>::Add(Int32)\npop\n"
            );
        }
        for i in 0..9 {
            body += &format!(
                "ldloca list\nldc.i4 {i}\ncall instance System.Collections.ArrayList<Int32>::get_Item(Int32)\nldc.i4 {i}\nbeq Item{i}\nfault \"lost item\"\nItem{i}:\n"
            );
        }
        body += "ldloca list\ncall instance System.Collections.ArrayList<Int32>::get_Capacity()";
        let result = program(&body, "Int32", "")
            .run(Limits {
                heap_objects: 3,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(result.value, Value::Int32(16));
        assert_eq!(result.memory.live_allocations(), 0);
    }
}

#[test]
fn managed_payloads_support_strings_bytes_void_and_records() {
    for (ty, value, expected) in [
        ("String", "ldstr \"Neo\"", Value::String("Neo".into())),
        ("Byte", "ldc.i4 257", Value::Byte(1)),
        ("Void", "ldvoid", Value::Void),
        (
            "Cell",
            "ldc.i4 42\nnewobj Cell",
            Value::Object {
                ty: neoclr::assembler::parse_type("Cell").unwrap(),
                fields: vec![Value::Int32(42)],
            },
        ),
    ] {
        let mut body = format!(
            ".local System.Collections.ArrayList<{ty}> list\nldc.i4 0\ncall System.Collections.ArrayList<{ty}>::Allocate(Int32)\nstloc list\n"
        );
        for _ in 0..5 {
            body += &format!(
                "ldloca list\n{value}\ncall instance System.Collections.ArrayList<{ty}>::Add({ty})\npop\n"
            );
        }
        body += &format!(
            "ldloca list\nldc.i4 4\ncall instance System.Collections.ArrayList<{ty}>::get_Item(Int32)"
        );
        assert_eq!(
            program(&body, ty, ".type Cell\n.field Value Int32\n.end")
                .run(Limits::default())
                .unwrap()
                .value,
            expected
        );
    }
}

#[test]
fn invalid_capacity_and_indices_fault_before_reading_spare_capacity() {
    for (body, expected) in [
        (
            "ldc.i4 -1\ncall System.Collections.ArrayList<Int32>::Allocate(Int32)\npop\nldc.i4 0",
            "capacity",
        ),
        (
            ".local System.Collections.ArrayList<Int32> list\nldc.i4 8\ncall System.Collections.ArrayList<Int32>::Allocate(Int32)\nstloc list\nldloca list\nldc.i4 0\ncall instance System.Collections.ArrayList<Int32>::get_Item(Int32)",
            "index",
        ),
    ] {
        assert!(
            program(body, "Int32", "")
                .run(Limits::default())
                .unwrap_err()
                .message
                .contains(expected)
        );
    }
}

#[test]
fn reference_elements_preserve_identity_and_gc_reachability_across_growth() {
    let extra = ".type Foo\n.field Age Int32\n.end\n.function Make() -> System.Collections.ArrayList<Foo&>\n.local System.Collections.ArrayList<Foo&> list\nldc.i4 0\ncall System.Collections.ArrayList<Foo&>::Allocate(Int32)\nstloc list\nldloca list\nldc.i4 40\nnewobj Foo\nheap.new\ncall instance System.Collections.ArrayList<Foo&>::Add(Foo&)\npop\nldloc list\nret\n.end";
    let body = ".local System.Collections.ArrayList<Foo&> list\n.local Foo& alias\ncall Make()\nstloc list\nldloca list\nldc.i4 0\ncall instance System.Collections.ArrayList<Foo&>::get_Item(Int32)\nstloc alias\nldloc alias\nldflda Foo::Age\nldc.i4 42\nstobj Int32\nldloca list\nldloc alias\ncall instance System.Collections.ArrayList<Foo&>::Add(Foo&)\npop\nldloca list\nldc.i4 1\ncall instance System.Collections.ArrayList<Foo&>::get_Item(Int32)\nldobj Foo\nldfld Foo::Age";
    let result = program(body, "Int32", extra)
        .run(Limits {
            heap_objects: 3,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.memory.live_allocations(), 0);
    assert_eq!(result.heap.len(), 0);
}

#[test]
fn frame_reference_elements_cannot_escape_into_managed_backing_storage() {
    let body = ".local Int32 value\n.local System.Collections.ArrayList<Int32&> list\nldc.i4 42\nstloc value\nldc.i4 1\ncall System.Collections.ArrayList<Int32&>::Allocate(Int32)\nstloc list\nldloca list\nldloca value\ncall instance System.Collections.ArrayList<Int32&>::Add(Int32&)";
    let fault = program(body, "Void", "")
        .run(Limits::default())
        .unwrap_err();
    assert!(fault.message.contains("frame-backed"), "{fault}");
}

#[test]
fn descriptor_copies_share_array_but_growth_detaches_the_copy() {
    let body = ".local System.Collections.ArrayList<Int32> list\n.local System.Collections.ArrayList<Int32> copy\nldc.i4 1\ncall System.Collections.ArrayList<Int32>::Allocate(Int32)\nstloc list\nldloca list\nldc.i4 1\ncall instance System.Collections.ArrayList<Int32>::Add(Int32)\npop\nldloc list\nstloc copy\nldloca copy\nldc.i4 0\nldc.i4 2\ncall instance System.Collections.ArrayList<Int32>::set_Item(Int32,Int32)\npop\nldloca list\nldc.i4 0\ncall instance System.Collections.ArrayList<Int32>::get_Item(Int32)\nldc.i4 2\nbeq Shared\nfault \"copy lost shared buffer\"\nShared:\nldloca copy\nldc.i4 3\ncall instance System.Collections.ArrayList<Int32>::Add(Int32)\npop\nldloca copy\nldc.i4 0\nldc.i4 4\ncall instance System.Collections.ArrayList<Int32>::set_Item(Int32,Int32)\npop\nldloca list\ncall instance System.Collections.ArrayList<Int32>::get_Count()\nldc.i4 1\nbeq Independent\nfault \"count was shared\"\nIndependent:\nldloca list\nldc.i4 0\ncall instance System.Collections.ArrayList<Int32>::get_Item(Int32)";
    assert_eq!(
        program(body, "Int32", "")
            .run(Limits::default())
            .unwrap()
            .value,
        Value::Int32(2)
    );
}

#[test]
fn replacing_a_reference_element_does_not_retain_it_in_spare_capacity() {
    let body = ".local System.Collections.ArrayList<Int32&> list\nldc.i4 8\ncall System.Collections.ArrayList<Int32&>::Allocate(Int32)\nstloc list\nldloca list\nldc.i4 1\nheap.new\ncall instance System.Collections.ArrayList<Int32&>::Add(Int32&)\npop\nldloca list\nldc.i4 0\nldc.i4 42\nheap.new\ncall instance System.Collections.ArrayList<Int32&>::set_Item(Int32,Int32&)\npop\nldloc list";
    let result = program(body, "System.Collections.ArrayList<Int32&>", "")
        .run(Limits::default())
        .unwrap();
    assert_eq!(result.heap.len(), 2);
    assert_eq!(result.heap.reclaimed_objects(), 1);
}

#[test]
fn neo_forwards_reference_elements_and_automatically_accesses_the_target() {
    let il = neoclr::frontend::lower_to_il("record Foo(Age: int)\nfunc Append(list: System.Collections.ArrayList<Foo&>&, item: Foo&) -> int { list.Add(item); let first = list.get_Item(0); first.Age = first.Age + 2; return first.Age }\nfunc Main() -> int { return 0 }").unwrap();
    let helpers = &il[..il.rfind(".function Main(").unwrap()];
    let module = assemble(&format!("{helpers}\n.function Main() -> Int32\n.local System.Collections.ArrayList<Foo&> list\nldc.i4 0\ncall System.Collections.ArrayList<Foo&>::Allocate(Int32)\nstloc list\nldloca list\nldc.i4 40\nnewobj Foo\nheap.new\ncall Append(System.Collections.ArrayList<Foo&>&,Foo&)\nret\n.end")).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}
