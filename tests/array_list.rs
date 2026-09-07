use neoclr::{Limits, LoadedProgram, Value, assemble, load};

fn execute(body: &str, returns: &str, extra: &str) -> Result<neoclr::Execution, neoclr::Fault> {
    let module = assemble(&format!(
        ".module App\n.entry Main\n{extra}\n.function Main() -> {returns}\n{body}\nret\n.end"
    ))?;
    let program = LoadedProgram::new(&load(&serde_json::to_string(&module).unwrap())?)?;
    program.verify()?;
    program.run(Limits::default())
}

#[test]
fn sample_grows_through_an_alias_and_releases_both_allocations() {
    let module = assemble(include_str!("../examples/array_list.neoil")).unwrap();
    let program =
        LoadedProgram::new(&load(&serde_json::to_string(&module).unwrap()).unwrap()).unwrap();
    program.verify().unwrap();
    let result = program.run(Limits::default()).unwrap();
    assert_eq!(
        result.output,
        ["ArrayList count:", "5", "0", "1", "4", "9", "16"]
    );
    assert_eq!(result.memory.live_allocations(), 0);
}

#[test]
fn empty_zero_capacity_and_repeated_growth_preserve_items_and_capacity() {
    for initial in [0, 1, 4] {
        let mut body = format!(
            ".local System.Collections.ArrayList<Int32> list\nldc.i4 {initial}\ncall System.Collections.ArrayList<Int32>::Allocate(Int32)\nstloc list\nldloc list\ncall instance System.Collections.ArrayList<Int32>::get_Count()\nldc.i4 0\nbeq Empty\nfault \"not empty\"\nEmpty:\n"
        );
        for index in 0..9 {
            body += &format!(
                "ldloc list\nldc.i4 {index}\ncall instance System.Collections.ArrayList<Int32>::Add(Int32)\npop\n"
            );
        }
        for index in 0..9 {
            body += &format!(
                "ldloc list\nldc.i4 {index}\ncall instance System.Collections.ArrayList<Int32>::get_Item(Int32)\nldc.i4 {index}\nbeq Item{index}\nfault \"lost item\"\nItem{index}:\n"
            );
        }
        body += "ldloc list\ncall instance System.Collections.ArrayList<Int32>::get_Capacity()\nldloc list\ncall instance System.Collections.ArrayList<Int32>::Free()\npop";
        let result = execute(&body, "Int32", "").unwrap();
        assert_eq!(result.value, Value::Int32(16));
        assert_eq!(result.memory.live_allocations(), 0);
    }
}

#[test]
fn aliases_share_indexed_mutation_and_count() {
    let body = ".local System.Collections.ArrayList<Int32> list\n.local System.Collections.ArrayList<Int32> alias\nldc.i4 0\ncall System.Collections.ArrayList<Int32>::Allocate(Int32)\nstloc list\nldloc list\nstloc alias\nldloc alias\nldc.i4 1\ncall instance System.Collections.ArrayList<Int32>::Add(Int32)\npop\nldloc list\nldc.i4 0\nldc.i4 42\ncall instance System.Collections.ArrayList<Int32>::set_Item(Int32,Int32)\npop\nldloc alias\nldc.i4 0\ncall instance System.Collections.ArrayList<Int32>::get_Item(Int32)\nldloc list\ncall instance System.Collections.ArrayList<Int32>::Free()\npop";
    assert_eq!(execute(body, "Int32", "").unwrap().value, Value::Int32(42));
}

#[test]
fn native_payload_storage_supports_byte_void_and_records() {
    for (ty, value, expected) in [
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
                "ldloc list\n{value}\ncall instance System.Collections.ArrayList<{ty}>::Add({ty})\npop\n"
            );
        }
        body += &format!(
            "ldloc list\nldc.i4 4\ncall instance System.Collections.ArrayList<{ty}>::get_Item(Int32)\nldloc list\ncall instance System.Collections.ArrayList<{ty}>::Free()\npop"
        );
        let result = execute(&body, ty, ".type Cell\n.field Value Int32\n.end").unwrap();
        assert_eq!(result.value, expected);
        assert_eq!(result.memory.live_allocations(), 0);
    }
}

#[test]
fn invalid_capacity_indices_expired_aliases_and_unsupported_layouts_fault() {
    let prefix = ".local System.Collections.ArrayList<Int32> list\nldc.i4 2\ncall System.Collections.ArrayList<Int32>::Allocate(Int32)\nstloc list\n";
    for (body, message) in [
        (
            "ldc.i4 -1\ncall System.Collections.ArrayList<Int32>::Allocate(Int32)\npop\nldvoid"
                .to_string(),
            "capacity",
        ),
        (
            format!(
                "{prefix}ldloc list\nldc.i4 0\ncall instance System.Collections.ArrayList<Int32>::get_Item(Int32)\npop\nldvoid"
            ),
            "index",
        ),
        (
            format!(
                "{prefix}ldloc list\nldc.i4 -1\nldc.i4 3\ncall instance System.Collections.ArrayList<Int32>::set_Item(Int32,Int32)"
            ),
            "index",
        ),
        (
            format!(
                "{prefix}ldloc list\ncall instance System.Collections.ArrayList<Int32>::Free()\npop\nldloc list\ncall instance System.Collections.ArrayList<Int32>::get_Count()\npop\nldvoid"
            ),
            "use after free",
        ),
        (
            "ldc.i4 0\ncall System.Collections.ArrayList<String>::Allocate(Int32)\npop\nldvoid"
                .into(),
            "layout",
        ),
    ] {
        let fault = execute(&body, "Void", "").unwrap_err();
        assert!(fault.message.contains(message), "{fault}");
    }
}

#[test]
fn empty_lists_release_storage_and_bounds_use_count_not_capacity() {
    let empty = ".local System.Collections.ArrayList<Int32> list\nldc.i4 8\ncall System.Collections.ArrayList<Int32>::Allocate(Int32)\nstloc list\nldloc list\ncall instance System.Collections.ArrayList<Int32>::Free()";
    let result = execute(empty, "Void", "").unwrap();
    assert_eq!(result.memory.live_allocations(), 0);
    let body = ".local System.Collections.ArrayList<Int32> list\nldc.i4 8\ncall System.Collections.ArrayList<Int32>::Allocate(Int32)\nstloc list\nldloc list\nldc.i4 42\ncall instance System.Collections.ArrayList<Int32>::Add(Int32)\npop\nldloc list\nldc.i4 1\ncall instance System.Collections.ArrayList<Int32>::get_Item(Int32)";
    let fault = execute(body, "Int32", "").unwrap_err();
    assert!(fault.message.contains("index out of range"), "{fault}");
}

#[test]
fn freeing_pointer_elements_does_not_free_their_targets() {
    let body = ".local Int32* target\n.local System.Collections.ArrayList<Int32*> list\nldc.i4 1\nheap.alloc Int32\nstloc target\nldloc target\nldc.i4 42\nstobj Int32\nldc.i4 0\ncall System.Collections.ArrayList<Int32*>::Allocate(Int32)\nstloc list\nldloc list\nldloc target\ncall instance System.Collections.ArrayList<Int32*>::Add(Int32*)\npop\nldloc list\ncall instance System.Collections.ArrayList<Int32*>::Free()\npop\nldloc target\nldobj Int32\nldloc target\nheap.free\npop";
    let result = execute(body, "Int32", "").unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.memory.live_allocations(), 0);
}
