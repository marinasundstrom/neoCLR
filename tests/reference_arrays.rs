use neoclr::{Limits, Value, assemble, run, verify};
fn module(body: &str) -> neoclr::Module {
    assemble(&format!(".module ReferenceArrays\n.entry Main\n{body}")).unwrap()
}

#[test]
fn aliases_share_elements_but_rebinding_preserves_old_allocation() {
    let m = module(
        r#"
.function Main() -> Int32
.local arrayref<Int32> first
.local arrayref<Int32> alias
ldc.i4 1
newarr Int32
stloc first
ldloc first
stloc alias
ldloc alias
ldc.i4 0
ldc.i4 42
stelem Int32
ldc.i4 2
newarr Int32
stloc alias
ldloc first
ldc.i4 0
ldelem Int32
ret
.end
"#,
    );
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(42));
}

#[test]
fn generic_constructor_defaults_array_field_then_assigns_buffer() {
    let m = module(
        r#"
.type class Buffer<T>
.field Items arrayref<T>
.method instance .ctor(Int32 count) -> noresult
ldarg this
ldfld Buffer<T>::Items
dup
ref.eq
pop
ldarg this
ldarg count
newarr T
stfld Buffer<T>::Items
ret
.end
.end
.function Main() -> Int32
.local Buffer<Int32> buffer
ldc.i4 1
newobj instance Buffer<Int32>::.ctor(Int32)
stloc buffer
ldloc buffer
ldfld Buffer<Int32>::Items
ldc.i4 0
ldc.i4 42
stelem Int32
ldloc buffer
ldfld Buffer<Int32>::Items
ldc.i4 0
ldelem Int32
ret
.end
"#,
    );
    let restored = serde_json::from_str(&serde_json::to_string(&m).unwrap()).unwrap();
    verify(&restored).unwrap();
    assert_eq!(
        run(&restored, Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn jagged_arrays_default_inner_references_to_null() {
    let m = module(
        r#"
.function Main() -> arrayref<Int32>
ldc.i4 2
newarr arrayref<Int32>
ldc.i4 0
ldelem arrayref<Int32>
ret
.end
"#,
    );
    verify(&m).unwrap();
    let result = run(&m, Limits::default()).unwrap();
    assert_eq!(
        result.value,
        Value::NullObjectReference(neoclr::assembler::parse_type("arrayref<Int32>").unwrap())
    );
    assert_eq!(result.heap.statistics().allocated_objects, 1);
}

#[test]
fn element_address_roots_returned_array_across_gc() {
    let m = module(
        r#"
.function Main() -> Int32
.local Int32& item
call Make()
stloc item
ldc.i4 1
newarr Int32
pop
ldc.i4 1
newarr Int32
pop
ldloc item
ldobj Int32
ret
.end
.function Make() -> Int32&
.local arrayref<Int32> items
ldc.i4 1
newarr Int32
stloc items
ldloc items
ldc.i4 0
ldc.i4 42
stelem Int32
ldloc items
ldc.i4 0
ldelema Int32
ret
.end
"#,
    );
    verify(&m).unwrap();
    let result = run(
        &m,
        Limits {
            heap_objects: 2,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert!(result.heap.statistics().collections > 0);
}

#[test]
fn null_array_operations_fault_without_verifier() {
    for (operation, returns) in [
        ("ldlen", "UIntPtr"),
        ("ldc.i4 0\nldelem Int32", "Int32"),
        ("ldc.i4 0\nldelema Int32", "Int32&"),
        ("ldc.i4 0\nldc.i4 42\nstelem Int32", "noresult"),
    ] {
        let m = module(&format!(
            ".function Main() -> {returns}\n.local arrayref<Int32> items\nldloca items\ninitobj arrayref<Int32>\nldloc items\n{operation}\nret\n.end"
        ));
        verify(&m).unwrap();
        assert!(
            run(&m, Limits::default())
                .unwrap_err()
                .message
                .contains("null array reference")
        );
    }
}

#[test]
fn exact_array_cast_preserves_identity_and_wrong_element_cast_faults() {
    let m = module(
        ".function Main() -> Boolean\nldc.i4 1\nnewarr Int32\ndup\ncastclass arrayref<Int32>\nref.eq\nret\n.end",
    );
    verify(&m).unwrap();
    assert_eq!(
        run(&m, Limits::default()).unwrap().value,
        Value::Boolean(true)
    );
    let m = module(
        ".function Main() -> arrayref<Boolean>\nldc.i4 1\nnewarr Int32\ncastclass arrayref<Boolean>\nret\n.end",
    );
    verify(&m).unwrap();
    assert!(run(&m, Limits::default()).is_err());
}

#[test]
fn reference_array_is_not_an_owned_array_or_native_payload() {
    for target in ["Int32[]", "Int32[]&"] {
        let m = module(&format!(
            ".function Main() -> {target}\nldc.i4 1\nnewarr Int32\nret\n.end"
        ));
        assert!(verify(&m).is_err());
        assert!(run(&m, Limits::default()).is_err());
    }
    let m = module(".function Main() -> arrayref<Int32>\nldc.i4 1\nnewarr Int32\nret\n.end");
    verify(&m).unwrap();
    let result = run(&m, Limits::default()).unwrap();
    let Value::ObjectReference(reference) = result.value else {
        panic!("object reference required")
    };
    assert_eq!(reference.concrete_type(), reference.target());
    assert!(neoclr::memory::layout(&m, reference.target()).is_err());
}

#[test]
fn array_reference_slot_requires_explicit_load_and_supports_reset() {
    let m = module(
        r#"
.function Main() -> Int32
.local arrayref<Int32> items
.local arrayref<Int32> alias
ldc.i4 1
newarr Int32
stloc items
ldloca items
ldobj arrayref<Int32>
stloc alias
ldloca items
initobj arrayref<Int32>
ldloc alias
ldc.i4 0
ldelem Int32
ret
.end
"#,
    );
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(0));
    let m = module(
        ".function Main() -> UIntPtr\n.local arrayref<Int32> items\nldc.i4 1\nnewarr Int32\nstloc items\nldloca items\nldlen\nret\n.end",
    );
    assert!(verify(&m).is_err());
    assert!(run(&m, Limits::default()).is_err());
}

#[test]
fn array_reference_identity_is_distinct_and_substitutes_in_metadata() {
    let m = module(".function Main() -> arrayref<Int32>\nldc.i4 1\nnewarr Int32\nret\n.end");
    let program = neoclr::LoadedProgram::new(&m).unwrap();
    let ty = neoclr::assembler::parse_type("arrayref<Int32>").unwrap();
    let owned = neoclr::assembler::parse_type("Int32[]").unwrap();
    assert_ne!(
        program.resolve_type_identity(&ty).unwrap(),
        program.resolve_type_identity(&owned).unwrap()
    );
    assert_eq!(
        ty,
        neoclr::assembler::parse_type("arrayref<!0>")
            .unwrap()
            .substitute_type_parameters(&[neoclr::metadata::Type::Int32])
            .unwrap()
    );
    let result = program.run(Limits::default()).unwrap();
    assert_eq!(result.value.ty(), ty);
}

#[test]
fn reference_arrays_fail_notreference_constraint() {
    use neoclr::metadata::{ConstraintKind, GenericConstraint};
    let mut m = module(
        r#"
.function Main() -> arrayref<Int32>
ldc.i4 1
newarr Int32
call Identity<arrayref<Int32>>(arrayref<Int32>)
ret
.end
.function Identity<T>(T) -> T
ldarg 0
ret
.end
"#,
    );
    m.functions
        .iter_mut()
        .find(|f| f.name == "Identity")
        .unwrap()
        .generic_constraints
        .push(GenericConstraint {
            parameter: 0,
            kind: ConstraintKind::NotReference,
        });
    assert!(verify(&m).is_err());
    assert!(run(&m, Limits::default()).is_err());
}

#[test]
fn interface_objects_survive_in_jagged_array_across_gc() {
    let m = module(
        r#"
.interface Read
.method instance Get() -> Int32
.end
.end
.type class Cell
.implements Read
.field Value Int32
.method instance Get() -> Int32
ldarg this
ldfld Cell::Value
ret
.end
.end
.function Main() -> Int32
.local arrayref<arrayref<Read>> outer
call Make()
stloc outer
ldc.i4 1
newarr Int32
pop
ldc.i4 1
newarr Int32
pop
ldloc outer
ldc.i4 0
ldelem arrayref<Read>
ldc.i4 0
ldelem Read
callvirt instance Read::Get()
ret
.end
.function Make() -> arrayref<arrayref<Read>>
.local arrayref<Read> inner
.local arrayref<arrayref<Read>> outer
ldc.i4 1
newarr Read
stloc inner
ldloc inner
ldc.i4 0
ldc.i4 42
newobj Cell
castclass Read
stelem Read
ldc.i4 1
newarr arrayref<Read>
stloc outer
ldloc outer
ldc.i4 0
ldloc inner
stelem arrayref<Read>
ldloc outer
ret
.end
"#,
    );
    verify(&m).unwrap();
    let result = run(
        &m,
        Limits {
            heap_objects: 4,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert!(result.heap.statistics().collections > 0);
}

#[test]
fn reflection_recognizes_ordinary_array_shape() {
    let m = module(
        ".function Main() -> Boolean\nldtoken arrayref<Int32>\ncall System.Type::GetTypeFromHandle(System.RuntimeTypeHandle)\ncall instance System.Type::get_IsArray()\nret\n.end",
    );
    verify(&m).unwrap();
    assert_eq!(
        run(&m, Limits::default()).unwrap().value,
        Value::Boolean(true)
    );
}

#[test]
fn ordinary_array_operations_enforce_bounds_types_and_budgets() {
    for (body, returns) in [
        ("ldc.i4 1\nnewarr Int32\nldc.i4 1\nldelem Int32", "Int32"),
        (
            "ldc.i4 1\nnewarr Int32\nldc.i4 0\nldelem Boolean",
            "Boolean",
        ),
        (
            "ldc.i4 1\nnewarr Int32\nldc.i4 0\nldstr \"bad\"\nstelem Int32",
            "noresult",
        ),
        ("ldc.i4 -1\nnewarr Int32", "arrayref<Int32>"),
    ] {
        let m = module(&format!(".function Main() -> {returns}\n{body}\nret\n.end"));
        assert!(run(&m, Limits::default()).is_err());
    }
    let m = module(".function Main() -> arrayref<Int32>\nldc.i4 2\nnewarr Int32\nret\n.end");
    for limits in [
        Limits {
            heap_objects: 0,
            ..Limits::default()
        },
        Limits {
            array_elements: 1,
            ..Limits::default()
        },
    ] {
        assert!(run(&m, limits).is_err());
    }
}
