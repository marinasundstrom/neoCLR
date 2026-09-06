use neoclr::{Limits, assemble, load, memory::layout, metadata::Type, run};

fn record(directives: &str) -> neoclr::Module {
    assemble(&format!(
        ".module Test\n.type Packet\n{directives}\n.field Tag Byte\n.field Number Int32\n.end"
    ))
    .unwrap()
}

#[test]
fn packing_controls_field_offsets_record_alignment_and_stride() {
    for (pack, size, alignment, offset) in [
        (0, 8, 4, 4),
        (1, 5, 1, 1),
        (2, 6, 2, 2),
        (4, 8, 4, 4),
        (8, 8, 4, 4),
        (128, 8, 4, 4),
    ] {
        let module = record(&format!(".pack {pack}"));
        let result = layout(&module, &Type::Named("Packet".into())).unwrap();
        assert_eq!(
            (result.size, result.alignment, result.fields[1].offset),
            (size, alignment, offset)
        );
    }
}

#[test]
fn size_is_a_minimum_reservation_rounded_to_record_alignment() {
    for (directives, expected) in [
        (".size 1", 8),
        (".size 9", 12),
        (".pack 1\n.size 9", 9),
        (".pack 2\n.size 9", 10),
    ] {
        assert_eq!(
            layout(&record(directives), &Type::Named("Packet".into()))
                .unwrap()
                .size,
            expected
        );
    }
}

#[test]
fn nested_layout_retains_inner_offsets_while_outer_packing_controls_placement() {
    let module = assemble(".module Test\n.type Inner\n.field Tag Byte\n.field Number Int32\n.end\n.type Outer\n.pack 1\n.field Prefix Byte\n.field Nested Inner\n.end").unwrap();
    let result = layout(&module, &Type::Named("Outer".into())).unwrap();
    assert_eq!(
        (result.size, result.alignment, result.fields[1].offset),
        (9, 1, 1)
    );
    assert_eq!(result.fields[1].layout.fields[1].offset, 4);
}

#[test]
fn invalid_layout_metadata_is_rejected_without_a_layout_opcode() {
    for directives in [
        ".pack 3",
        ".pack 256",
        ".pack -1",
        ".size -1",
        ".size 2147483648",
        ".pack 1\n.pack 2",
        ".size 1\n.size 2",
    ] {
        assert!(assemble(&format!(".module Test\n.type Packet\n{directives}\n.end")).is_err());
    }
    assert!(assemble(".module Test\n.type System.Int32\n.pack 1\n.end").is_err());
    let mut json = serde_json::to_value(record("")).unwrap();
    json["types"][0]["packing"] = serde_json::json!(3);
    assert!(load(&json.to_string()).is_err());
    assert!(load(&serde_json::to_string(&record("")).unwrap()).is_ok());
}

#[test]
fn packed_field_addresses_still_require_natural_alignment_for_indirect_access() {
    let source = ".module Test\n.entry Main\n.type Packet\n.pack 1\n.field Tag Byte\n.field Number Int32\n.end\n.function Main() -> Int32\nsizeof Packet\nlocalloc\nptr.cast Packet\ndup\ninitobj Packet\nldflda Packet::Number\nldind.i4\nret\n.end";
    assert!(
        run(&assemble(source).unwrap(), Limits::default())
            .unwrap_err()
            .message
            .contains("misaligned")
    );
}

#[test]
fn sample_roundtrips_packed_storage_and_releases_allocations() {
    let module = assemble(include_str!("../examples/layout.neoil")).unwrap();
    let module = load(&serde_json::to_string(&module).unwrap()).unwrap();
    let execution = run(&module, Limits::default()).unwrap();
    assert_eq!(execution.output, ["42", "8"]);
    assert_eq!(execution.memory.live_allocations(), 0);
}
