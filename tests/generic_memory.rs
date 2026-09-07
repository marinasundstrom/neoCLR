use neoclr::{
    Limits, Value, assemble, assembler::parse_type, load, memory::layout, metadata::Type, run,
};

#[test]
fn sample_roundtrips_and_uses_generic_native_storage() {
    let module = assemble(include_str!("../examples/generic-memory.neoil")).unwrap();
    let loaded = load(&serde_json::to_string(&module).unwrap()).unwrap();
    assert_eq!(run(&loaded, Limits::default()).unwrap().output, ["42", "4"]);
}

#[test]
fn layout_substitutes_nested_fields_packing_and_minimum_size() {
    let module = assemble(".module Test\n.type Box<T>\n.field Value T\n.end\n.type Packed<T>\n.pack 1\n.size 9\n.field Flag Byte\n.field Value T\n.end").unwrap();
    let nested = layout(&module, &parse_type("Box<Box<Int32>>").unwrap()).unwrap();
    assert_eq!((nested.size, nested.alignment), (4, 4));
    assert_eq!(nested.fields[0].ty, parse_type("Box<Int32>").unwrap());
    let packed = layout(&module, &parse_type("Packed<Box<Int32>>").unwrap()).unwrap();
    assert_eq!((packed.size, packed.alignment), (9, 1));
    assert_eq!(packed.fields[1].offset, 1);
    assert_eq!(
        layout(&module, &parse_type("Box<Void>").unwrap())
            .unwrap()
            .size,
        0
    );
    assert_eq!(
        layout(&module, &parse_type("Box<Byte>").unwrap())
            .unwrap()
            .size,
        1
    );
    assert_eq!(module.types[0].fields[0].ty, Type::TypeParameter(0));
}

#[test]
fn generic_records_roundtrip_through_copy_and_initialization() {
    let source = ".module Test\n.entry Main\n.type Box<T>\n.field Value T\n.end\n.function Main() -> Int32\n.local Box<Box<Int32>>* a\n.local Box<Box<Int32>>* b\nldc.i4 1\nheap.alloc Box<Box<Int32>>\nstloc a\nldc.i4 1\nheap.alloc Box<Box<Int32>>\nstloc b\nldloc a\nldc.i4 42\nnewobj Box<Int32>\nnewobj Box<Box<Int32>>\nstobj Box<Box<Int32>>\nldloc b\nldloc a\ncpobj Box<Box<Int32>>\nldloc a\ninitobj Box<Box<Int32>>\nldloc b\nldobj Box<Box<Int32>>\nldfld 0\nldfld 0\nldloc a\nldobj Box<Box<Int32>>\nldfld 0\nldfld 0\nadd\nldloc a\nheap.free\npop\nldloc b\nheap.free\npop\nret\n.end";
    assert_eq!(
        run(&assemble(source).unwrap(), Limits::default())
            .unwrap()
            .value,
        Value::Int32(42)
    );
}

#[test]
fn loads_preserve_closed_identity_and_require_initialized_fields() {
    let source = ".module Test\n.entry Main\n.type Box<T>\n.field Value T\n.end\n.function Main() -> Box<Byte>\n.local Box<Byte>* p\nldc.i4 1\nheap.alloc Box<Byte>\nstloc p\nldloc p\nldc.i4 257\nnewobj Box<Byte>\nstobj Box<Byte>\nldloc p\nldobj Box<Byte>\nldloc p\nheap.free\npop\nret\n.end";
    assert_eq!(
        run(&assemble(source).unwrap(), Limits::default())
            .unwrap()
            .value,
        Value::Object {
            ty: parse_type("Box<Byte>").unwrap(),
            fields: vec![Value::Byte(1)]
        }
    );
    let uninitialized = source.replace(
        "ldloc p\nldc.i4 257\nnewobj Box<Byte>\nstobj Box<Byte>\n",
        "",
    );
    assert!(
        run(&assemble(&uninitialized).unwrap(), Limits::default())
            .unwrap_err()
            .message
            .contains("uninitialized")
    );
    let wrong = source.replace("newobj Box<Byte>", "newobj Box<Int32>");
    assert!(run(&assemble(&wrong).unwrap(), Limits::default()).is_err());
}

#[test]
fn packed_generic_fields_require_unaligned_access_when_needed() {
    let source = ".module Test\n.entry Main\n.type Packed<T>\n.pack 1\n.field Flag Byte\n.field Value T\n.end\n.function Main() -> Int32\n.local Packed<Int32>* p\nldc.i4 1\nheap.alloc Packed<Int32>\nstloc p\nldloc p\nldc.i4 0\nldc.i4 42\nnewobj Packed<Int32>\nstobj Packed<Int32>\nldloc p\nldflda Packed<Int32>::Value\nunaligned. 1\nldind.i4\nldloc p\nheap.free\npop\nret\n.end";
    assert_eq!(
        run(&assemble(source).unwrap(), Limits::default())
            .unwrap()
            .value,
        Value::Int32(42)
    );
    assert!(
        run(
            &assemble(&source.replace("unaligned. 1\n", "")).unwrap(),
            Limits::default()
        )
        .is_err()
    );
}

#[test]
fn recursive_pointer_fields_have_finite_layout_and_preserve_pointer_lifetime() {
    let module =
        assemble(".module Test\n.type Node<T>\n.field Next Node<T>*\n.field Value T\n.end")
            .unwrap();
    let native = layout(&module, &parse_type("Node<Int32>").unwrap()).unwrap();
    assert_eq!(native.fields[0].ty, parse_type("Node<Int32>*").unwrap());
    let source = ".module Test\n.entry Main\n.type Box<T>\n.field Value T\n.end\n.function Main() -> Int32\n.local Int32* target\n.local Box<Int32*>* storage\nldc.i4 1\nheap.alloc Int32\nstloc target\nldc.i4 1\nheap.alloc Box<Int32*>\nstloc storage\nldloc storage\nldloc target\nnewobj Box<Int32*>\nstobj Box<Int32*>\nldloc target\nheap.free\npop\nldloc storage\nldobj Box<Int32*>\nldfld 0\nldind.i4\nret\n.end";
    assert!(
        run(&assemble(source).unwrap(), Limits::default())
            .unwrap_err()
            .message
            .contains("use after free")
    );
}

#[test]
fn recursive_by_value_and_unsupported_layouts_fail_without_unbounded_expansion() {
    let module = assemble(".module Test\n.type Box<T>\n.field Value T\n.end\n.type Loop<T>\n.field Next Loop<T>\n.end\n.type Grow<T>\n.field Next Grow<Grow<T>>\n.end").unwrap();
    for name in [
        "Loop<Int32>",
        "Grow<Int32>",
        "Box<String>",
        "Box<System.Option<Int32>>",
        "Box<!0>",
        "Box",
        "Box<Int32,Int32>",
    ] {
        assert!(
            layout(&module, &parse_type(name).unwrap()).is_err(),
            "{name}"
        );
    }
    let source = ".module Test\n.type Box<T>\n.field Value T\n.end\n.function Main() -> Int32\nsizeof Box<Int32>\nret\n.end";
    let mut json = serde_json::to_value(assemble(source).unwrap()).unwrap();
    json["functions"][0]["body"][0]["arg"] =
        serde_json::to_value(parse_type("Box<String>").unwrap()).unwrap();
    assert!(load(&json.to_string()).is_err());
}

#[test]
fn zero_sized_generic_records_keep_their_type_in_native_storage() {
    let source = ".module Test\n.entry Main\n.type Box<T>\n.field Value T\n.end\n.function Main() -> Box<Void>\n.local Box<Void>* p\nldc.i4 1\nheap.alloc Box<Void>\nstloc p\nldloc p\nldvoid\nnewobj Box<Void>\nstobj Box<Void>\nldloc p\nldobj Box<Void>\nldloc p\nheap.free\npop\nret\n.end";
    assert_eq!(
        run(&assemble(source).unwrap(), Limits::default())
            .unwrap()
            .value,
        Value::Object {
            ty: parse_type("Box<Void>").unwrap(),
            fields: vec![Value::Void]
        }
    );
}
