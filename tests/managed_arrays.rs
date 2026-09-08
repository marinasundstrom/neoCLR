use neoclr::{Limits, LoadedProgram, Value, assemble};

fn program(extra: &str, body: &str, returns: &str) -> LoadedProgram {
    let module = assemble(&format!(
        ".module Arrays\n.entry Main\n{extra}\n.function Main() -> {returns}\n{body}\nret\n.end"
    ))
    .unwrap();
    let module = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    LoadedProgram::new(&module).unwrap()
}

#[test]
fn heap_arrays_have_default_elements_and_native_length_and_indices() {
    let p = program("", "ldc.i4 3\nconv.u\nnewarr Int32\nldlen", "UIntPtr");
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::UIntPtr(3));
    let p = program(
        "",
        "ldc.i4 3\nnewarr Int32\nldc.i4 2\nconv.i\nldelem Int32",
        "Int32",
    );
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(0));
}

#[test]
fn value_copies_are_independent_and_element_references_follow_replacement() {
    let p = program(
        "",
        ".local Int32[] a\n.local Int32[] b\n.local Int32& item\nldc.i4 2\nldc.i4 7\narray.create Int32\nstloc a\nldloc a\nstloc b\nldloca a\nldc.i4 1\nldelema Int32\nstloc item\nldc.i4 2\nldc.i4 42\narray.create Int32\nstloc a\nldloc item\nldobj Int32\nldloca b\nldc.i4 1\nldelem Int32\nadd",
        "Int32",
    );
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(49));
}

#[test]
fn only_element_reference_roots_array_and_transitive_heap_fields() {
    let p = program(
        ".type Holder\n.field Child Int32&\n.end\n.function Make() -> Holder&\nldc.i4 1\nldc.i4 42\nheap.new\nnewobj Holder\narray.create Holder\nheap.new\nldc.i4 0\nldelema Holder\nret\n.end",
        ".local Holder& item\ncall Make()\nstloc item\nldc.i4 0\nheap.new\npop\nldc.i4 0\nheap.new\npop\nldloc item\nldobj Holder\nldfld Holder::Child\nldobj Int32",
        "Int32",
    );
    p.verify().unwrap();
    let result = p
        .run(Limits {
            heap_objects: 3,
            ..Limits::default()
        })
        .unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.heap.reclaimed_objects(), 4);
}

#[test]
fn current_frame_element_reference_cannot_return_even_without_verification() {
    let p = program(
        "",
        ".local Int32[] a\nldc.i4 1\nldc.i4 0\narray.create Int32\nstloc a\nldloca a\nldc.i4 0\nldelema Int32",
        "Int32&",
    );
    assert!(p.verify().is_err());
    assert!(p.run(Limits::default()).is_err());
}

#[test]
fn bounds_and_invalid_operands_fault() {
    for index in [-1, 2] {
        for op in [
            "ldelem Int32",
            "ldelema Int32",
            "ldc.i4 7\nstelem Int32\nldc.i4 0",
        ] {
            let returns = if op == "ldelema Int32" {
                "Int32&"
            } else {
                "Int32"
            };
            let p = program(
                "",
                &format!("ldc.i4 2\nnewarr Int32\nldc.i4 {index}\n{op}"),
                returns,
            );
            assert!(
                p.run(Limits::default())
                    .unwrap_err()
                    .message
                    .contains("range")
            );
        }
    }
    for body in [
        "ldc.i4 1\nnewarr Int32\nldstr \"bad\"\nldelem Int32",
        "ldc.i4 1\nnewarr Int32\nldc.i4 0\nldelem String",
    ] {
        let p = program("", body, "Int32");
        assert!(p.verify().is_err());
        assert!(p.run(Limits::default()).is_err());
    }
}

#[test]
fn fixed_shape_prevents_invalidating_existing_element_paths() {
    let p = program(
        "",
        ".local Int32[] a\nldc.i4 2\nldc.i4 1\narray.create Int32\nstloc a\nldc.i4 1\nldc.i4 1\narray.create Int32\nstloc a\nldc.i4 0",
        "Int32",
    );
    p.verify().unwrap();
    assert!(
        p.run(Limits::default())
            .unwrap_err()
            .message
            .contains("equal lengths")
    );
}

#[test]
fn allocation_budgets_bound_copies_strings_and_lengths() {
    for body in [
        "ldc.i4 4\nldc.i4 0\narray.create Int32",
        "ldc.i4 2\nldc.i4 0\narray.create Int32\ndup\npop",
        "ldc.i4 -1\nnewarr Int32",
    ] {
        let p = program("", &format!("{body}\npop\nldc.i4 0"), "Int32");
        assert!(
            p.run(Limits {
                array_elements: 3,
                ..Limits::default()
            })
            .is_err()
        );
    }
    let p = program(
        "",
        "ldc.i4 2\nldstr \"a moderately long initializer\"\narray.create String\npop\nldc.i4 0",
        "Int32",
    );
    assert!(
        p.run(Limits {
            array_bytes: 1,
            ..Limits::default()
        })
        .is_err()
    );
    let p = program(
        "",
        "ldc.i4 0\nldstr \"Neo\"\narray.create String\nldlen",
        "UIntPtr",
    );
    assert_eq!(
        p.run(Limits {
            array_elements: 0,
            array_bytes: 0,
            ..Limits::default()
        })
        .unwrap()
        .value,
        Value::UIntPtr(0)
    );
}

#[test]
fn payload_pressure_collects_unreachable_arrays_before_faulting() {
    let p = program(
        "",
        "ldc.i4 2\nnewarr Int32\npop\nldc.i4 2\nnewarr Int32\nldlen",
        "UIntPtr",
    );
    let result = p
        .run(Limits {
            array_elements: 2,
            ..Limits::default()
        })
        .unwrap();
    assert_eq!(result.value, Value::UIntPtr(2));
    assert_eq!(result.heap.reclaimed_objects(), 2);
}

#[test]
fn generic_arrays_substitute_elements_and_normalize_small_scalar_loads() {
    let p = program(
        ".type Access<T>\n.method static First(T[]& values) -> T\nldarg values\nldc.i4 0\nldelem T\nret\n.end\n.end",
        "ldc.i4 2\nldc.i4 42\narray.create Byte\nheap.new\ncall Access<Byte>::First(Byte[]&)",
        "Byte",
    );
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Byte(42));
}

#[test]
fn direct_reference_elements_and_unsupported_default_values_are_rejected() {
    for element in ["Int32&", "String"] {
        let module = assemble(&format!(
            ".module Bad\n.entry Main\n.function Main() -> Int32\nldc.i4 0\nnewarr {element}\npop\nldc.i4 0\nret\n.end"
        ));
        assert!(module.is_err());
    }
}

#[test]
fn replacement_of_array_inside_record_preserves_shape() {
    let p = program(
        ".type Container\n.field Items Int32[]\n.end",
        ".local Container c\nldc.i4 2\nldc.i4 0\narray.create Int32\nnewobj Container\nstloc c\nldloca c\nldc.i4 1\nldc.i4 0\narray.create Int32\nnewobj Container\nstobj Container\nldc.i4 0",
        "Int32",
    );
    p.verify().unwrap();
    assert!(
        p.run(Limits::default())
            .unwrap_err()
            .message
            .contains("equal lengths")
    );
}

#[test]
fn reserved_capacity_faults_on_reads_and_accepts_typed_initialization() {
    for access in ["ldelem Int32", "ldelema Int32\nldobj Int32"] {
        let p = program(
            "",
            &format!("ldc.i4 2\narray.alloc Int32\nldc.i4 0\n{access}"),
            "Int32",
        );
        p.verify().unwrap();
        assert!(
            p.run(Limits::default())
                .unwrap_err()
                .message
                .contains("uninitialized")
        );
    }
    let p = program(
        "",
        ".local String[]& a\nldc.i4 4\narray.alloc String\nstloc a\nldloc a\nldc.i4 2\nldstr \"Neo\"\nstelem String\nldloc a\nldc.i4 2\nldelem String",
        "String",
    );
    p.verify().unwrap();
    assert_eq!(
        p.run(Limits::default()).unwrap().value,
        Value::String("Neo".into())
    );
}

#[test]
fn reference_elements_trace_targets_and_replacement_releases_old_roots() {
    let p = program(
        "",
        ".local Int32&[]& a\nldc.i4 1\narray.alloc Int32&\nstloc a\nldloc a\nldc.i4 0\nldc.i4 1\nheap.new\nstelem Int32&\nldloc a\nldc.i4 0\nldc.i4 42\nheap.new\nstelem Int32&\nldloc a",
        "Int32&[]&",
    );
    p.verify().unwrap();
    let result = p.run(Limits::default()).unwrap();
    assert_eq!(result.heap.len(), 2);
    assert_eq!(result.heap.reclaimed_objects(), 1);
}

#[test]
fn reserved_arrays_obey_budgets_and_cannot_deinitialize_existing_elements() {
    let p = program("", "ldc.i4 4\narray.alloc String\nldlen", "UIntPtr");
    assert!(
        p.run(Limits {
            array_elements: 3,
            ..Default::default()
        })
        .unwrap_err()
        .message
        .contains("budget")
    );
    let p = program(
        "",
        ".local Int32[]& a\nldc.i4 1\nnewarr Int32\nstloc a\nldloc a\nldc.i4 1\narray.alloc Int32\nldobj Int32[]\nstobj Int32[]\nldc.i4 0",
        "Int32",
    );
    p.verify().unwrap();
    assert!(
        p.run(Limits::default())
            .unwrap_err()
            .message
            .contains("cannot become uninitialized")
    );
}
