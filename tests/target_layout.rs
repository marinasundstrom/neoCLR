use neoclr::{
    LoadedProgram, assemble,
    assembler::parse_type,
    memory::{TargetLayout, layout, layout_for},
    metadata::Type,
};

fn narrow() -> TargetLayout {
    TargetLayout {
        pointer_size: 4,
        pointer_alignment: 4,
        int64_alignment: 4,
        single_alignment: 4,
        double_alignment: 4,
    }
}
fn wide() -> TargetLayout {
    TargetLayout {
        pointer_size: 8,
        pointer_alignment: 8,
        int64_alignment: 8,
        single_alignment: 4,
        double_alignment: 8,
    }
}

#[test]
fn pointer_and_native_integer_layouts_follow_the_explicit_target() {
    let module = assemble(".module App\n.type Packet\n.field Tag Byte\n.field Link Packet*\n.field Count UInt64\n.field Weight Double\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    for (target, size, alignment, offsets) in [
        (narrow(), 24, 4, vec![0, 4, 8, 16]),
        (wide(), 32, 8, vec![0, 8, 16, 24]),
    ] {
        let result = program
            .layout_of(&parse_type("[App]Packet").unwrap(), target)
            .unwrap();
        assert_eq!((result.size, result.alignment), (size, alignment));
        assert_eq!(
            result.fields.iter().map(|f| f.offset).collect::<Vec<_>>(),
            offsets
        );
        for ty in [Type::IntPtr, Type::UIntPtr, parse_type("Packet*").unwrap()] {
            let result = program.layout_of(&ty, target).unwrap();
            assert_eq!(
                (result.size, result.alignment),
                (
                    usize::from(target.pointer_size),
                    usize::from(target.pointer_alignment)
                )
            );
        }
    }
}

#[test]
fn scalar_alignment_is_independent_of_pointer_width() {
    let module = assemble(".module App\n.type Pair\n.field Tag Byte\n.field Weight Double\n.end\n.type Counter\n.field Tag Byte\n.field Count Int64\n.end\n.type Float\n.field Tag Byte\n.field Value Single\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let target = TargetLayout {
        double_alignment: 8,
        int64_alignment: 8,
        single_alignment: 2,
        ..narrow()
    };
    for ty in ["Pair", "Counter"] {
        let result = program.layout_of(&parse_type(ty).unwrap(), target).unwrap();
        assert_eq!(
            (result.size, result.alignment, result.fields[1].offset),
            (16, 8, 8)
        );
    }
    let result = program
        .layout_of(&parse_type("Float").unwrap(), target)
        .unwrap();
    assert_eq!(
        (result.size, result.alignment, result.fields[1].offset),
        (6, 2, 2)
    );
}

#[test]
fn packing_minimum_size_and_nested_generics_share_the_target() {
    let module = assemble(".module App\n.type Box<T>\n.field Tag Byte\n.field Value T\n.end\n.type Packed<T>\n.pack 1\n.size 12\n.field Value Box<T>\n.field Tail Byte\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    for (target, size, nested_size, nested_offset) in [(narrow(), 12, 8, 4), (wide(), 17, 16, 8)] {
        let result = program
            .layout_of(&parse_type("Packed<IntPtr>").unwrap(), target)
            .unwrap();
        assert_eq!((result.size, result.alignment), (size, 1));
        assert_eq!(result.fields[1].offset, nested_size);
        assert_eq!(result.fields[0].layout.fields[1].offset, nested_offset);
    }
    let unit = program
        .layout_of(&parse_type("Box<Void>").unwrap(), wide())
        .unwrap();
    assert_eq!(
        (unit.size, unit.alignment, unit.fields[1].layout.size),
        (1, 1, 0)
    );
}

#[test]
fn invalid_descriptors_fail_even_for_layouts_that_do_not_use_the_invalid_scalar() {
    let module = assemble(".module App").unwrap();
    for target in [
        TargetLayout {
            pointer_size: 0,
            ..wide()
        },
        TargetLayout {
            pointer_size: 16,
            ..wide()
        },
        TargetLayout {
            pointer_alignment: 0,
            ..wide()
        },
        TargetLayout {
            pointer_alignment: 3,
            ..wide()
        },
        TargetLayout {
            pointer_alignment: 8,
            ..narrow()
        },
        TargetLayout {
            int64_alignment: 16,
            ..wide()
        },
        TargetLayout {
            single_alignment: 8,
            ..wide()
        },
        TargetLayout {
            double_alignment: 3,
            ..wide()
        },
    ] {
        assert!(layout_for(&module, &Type::Void, target).is_err());
    }
}

#[test]
fn closedness_scopes_and_unsupported_storage_keep_existing_checks() {
    let module = assemble(
        ".module App\n.type Box<T>\n.field Value T\n.end\n.type Loop\n.field Next Loop\n.end",
    )
    .unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    for ty in [
        "Box<!0>",
        "[Wrong]Box<Int32>",
        "Loop",
        "String",
        "Ref<Int32>",
        "System.Option<Int32>",
        "System.Result<Int32,Error>",
    ] {
        assert!(
            program
                .layout_of(&parse_type(ty).unwrap(), narrow())
                .is_err(),
            "{ty}"
        );
    }
    assert_eq!(
        program
            .layout_of(&parse_type("Loop*").unwrap(), narrow())
            .unwrap()
            .size,
        4
    );
}

#[test]
fn host_layout_and_execution_remain_consistent_after_foreign_layout_queries() {
    let module = assemble(".module App\n.entry Main\n.type Pair\n.field Flag Byte\n.field Pointer IntPtr\n.end\n.function Main() -> Int32\nsizeof Pair\nret\n.end").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    for target in [narrow(), wide()] {
        program
            .layout_of(&parse_type("Pair").unwrap(), target)
            .unwrap();
    }
    let direct = layout(&module, &parse_type("Pair").unwrap()).unwrap();
    let explicit = program
        .layout_of(&parse_type("Pair").unwrap(), TargetLayout::host())
        .unwrap();
    assert_eq!(
        (direct.size, direct.alignment, direct.fields[1].offset),
        (explicit.size, explicit.alignment, explicit.fields[1].offset)
    );
    assert_eq!(
        program.run(neoclr::Limits::default()).unwrap().value,
        neoclr::Value::Int32(direct.size as i32)
    );
}
