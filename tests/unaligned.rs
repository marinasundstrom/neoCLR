use neoclr::{Limits, Value, assemble, load, run};

fn program(body: &str) -> neoclr::Module {
    assemble(&format!(".module Test\n.entry Main\n.function Main() -> Int32\n.local Int32* pointer\nldc.i4 12\nlocalloc\nldc.i4 1\nptr.add\nptr.cast Int32\nstloc pointer\n{body}\nret\n.end")).unwrap()
}

#[test]
fn unaligned_indirect_and_typed_accesses_roundtrip() {
    for (store, read) in [("stind.i4", "ldind.i4"), ("stobj Int32", "ldobj Int32")] {
        let module = program(&format!(
            "ldloc pointer\nldc.i4 42\nunaligned. 1\n{store}\nldloc pointer\nunaligned. 1\n{read}"
        ));
        let module = load(&serde_json::to_string(&module).unwrap()).unwrap();
        assert_eq!(
            run(&module, Limits::default()).unwrap().value,
            Value::Int32(42)
        );
    }
}

#[test]
fn prefix_only_changes_one_access_and_preserves_alignment_promises() {
    for read in [
        "ldind.i4",
        "unaligned. 2\nldind.i4",
        "unaligned. 4\nldind.i4",
    ] {
        let module = program(&format!(
            "ldloc pointer\nldc.i4 42\nunaligned. 1\nstind.i4\nldloc pointer\n{read}"
        ));
        assert!(
            run(&module, Limits::default())
                .unwrap_err()
                .message
                .contains("misaligned")
        );
    }
    for alignment in [2, 4] {
        let module = assemble(&format!(".module Test\n.entry Main\n.function Main() -> Int32\n.local Int64* pointer\nldc.i4 16\nlocalloc\nldc.i4 {alignment}\nptr.add\nptr.cast Int64\nstloc pointer\nldloc pointer\nldc.i8 42\nunaligned. {alignment}\nstind.i8\nldloc pointer\nunaligned. {alignment}\nldind.i8\nconv.i4\nret\n.end")).unwrap();
        assert_eq!(
            run(&module, Limits::default()).unwrap().value,
            Value::Int32(42)
        );
    }
}

#[test]
fn unaligned_access_retains_initialization_bounds_and_null_checks() {
    for (body, message) in [
        ("ldloc pointer\nunaligned. 1\nldind.i4", "uninitialized"),
        (
            "ldloc pointer\nldc.i4 10\nptr.add\nunaligned. 1\nldind.i4",
            "out of bounds",
        ),
        ("ptr.null Int32\nunaligned. 1\nldind.i4", "null pointer"),
        (
            "ldc.i4 1\nheap.alloc Int32\ndup\nheap.free\npop\nunaligned. 1\nldind.i4",
            "use after free",
        ),
    ] {
        assert!(
            run(&program(body), Limits::default())
                .unwrap_err()
                .message
                .contains(message)
        );
    }
}

#[test]
fn invalid_prefix_values_and_placement_are_rejected_in_source_and_metadata() {
    for body in [
        "unaligned. 0\nldind.i4",
        "unaligned. 3\nldind.i4",
        "unaligned. 8\nldind.i4",
        "unaligned. 1",
        "unaligned. 1\nret",
        "unaligned. 1\nunaligned. 1\nldind.i4",
        "unaligned. 1\ninitobj Int32",
        "unaligned. 1\nldfld 0",
    ] {
        assert!(
            assemble(&format!(
                ".module Test\n.function F() -> Void\n{body}\n.end"
            ))
            .is_err(),
            "{body}"
        );
    }
    let mut json = serde_json::to_value(program("ldloc pointer\nunaligned. 1\nldind.i4")).unwrap();
    let instructions = json["functions"][0]["body"].as_array_mut().unwrap();
    let prefix = instructions
        .iter_mut()
        .find(|op| op["op"] == "unaligned.")
        .unwrap();
    prefix["arg"] = serde_json::json!(3);
    assert!(load(&json.to_string()).is_err());
}

#[test]
fn branches_must_target_prefix_not_the_modified_instruction() {
    for branch in [
        "br Access",
        "ldc.bool true\nbrtrue Access",
        "ldc.i4 0\nswitch (Access)",
        "ldc.i4 1\nldc.i4 1\nbeq Access",
    ] {
        assert!(assemble(&format!(".module Test\n.function F() -> Int32\n{branch}\nunaligned. 1\nAccess:\nldind.i4\nret\n.end")).is_err());
    }
    let module = program(
        "ldloc pointer\nldc.i4 42\nunaligned. 1\nstind.i4\nldloc pointer\nbr Access\nAccess:\nunaligned. 1\nldind.i4",
    );
    assert_eq!(
        run(&module, Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn prefixed_block_operations_accept_byte_ranges() {
    let module = program(
        "ldloc pointer\nldc.i4 0\nldc.i4 4\nunaligned. 1\ninitblk\nldloc pointer\nldloc pointer\nldc.i4 4\nunaligned. 1\ncpblk\nldloc pointer\nunaligned. 1\nldind.i4",
    );
    assert_eq!(
        run(&module, Limits::default()).unwrap().value,
        Value::Int32(0)
    );
}
