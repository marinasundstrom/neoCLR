use neoclr::{Limits, Value, assemble, load, metadata::Type, run};

fn module(ty: &str, body: &str) -> neoclr::Module {
    assemble(&format!(
        ".module Test\n.entry Main\n.function Main() -> {ty}\n{body}\nret\n.end"
    ))
    .unwrap()
}
fn eval(ty: &str, body: &str) -> Value {
    run(&module(ty, body), Limits::default()).unwrap().value
}

#[test]
fn aliases_have_canonical_identity_and_native_layout() {
    for (names, ty) in [
        (["nint", "IntPtr", "System.IntPtr"], Type::IntPtr),
        (["nuint", "UIntPtr", "System.UIntPtr"], Type::UIntPtr),
    ] {
        for name in names {
            assert_eq!(Type::from_name(name), ty);
            assert_eq!(
                eval("int32", &format!("sizeof {name}")),
                Value::Int32(size_of::<usize>() as i32)
            );
            assert_eq!(
                eval("int32", &format!("alignof {name}")),
                Value::Int32(align_of::<usize>() as i32)
            );
        }
        assert!(
            neoclr::library::system()
                .unwrap()
                .type_definition(&ty)
                .is_some()
        );
    }
}

#[test]
fn conversions_sign_extend_zero_extend_and_truncate() {
    assert_eq!(eval("nint", "ldc.i4 -1\nconv.i"), Value::IntPtr(-1));
    assert_eq!(
        eval("nuint", "ldc.i4 -1\nconv.u"),
        Value::UIntPtr(u32::MAX as usize)
    );
    assert_eq!(
        eval("nuint", "ldc.i4 -1\nconv.i\nconv.u"),
        Value::UIntPtr(usize::MAX)
    );
    assert_eq!(
        eval("int32", "ldc.i4 -1\nconv.i\nconv.u\nconv.i4"),
        Value::Int32(-1)
    );
    assert_eq!(eval("nint", "ptr.null Void\nconv.i"), Value::IntPtr(0));
}

#[test]
fn arithmetic_uses_native_width_and_opcode_signedness() {
    for (op, expected) in [
        ("add", 45),
        ("sub", 39),
        ("mul", 126),
        ("div", 14),
        ("add.ovf", 45),
        ("sub.ovf", 39),
        ("mul.ovf", 126),
    ] {
        assert_eq!(
            eval(
                "nint",
                &format!("ldc.i4 42\nconv.i\nldc.i4 3\nconv.i\n{op}")
            ),
            Value::IntPtr(expected)
        );
    }
    assert_eq!(
        eval("nuint", "ldc.i4 -1\nconv.i\nconv.u\nldc.i4 1\nconv.u\nadd"),
        Value::UIntPtr(0)
    );
    // Signed opcodes reinterpret native unsigned bits as signed, as CIL does.
    assert_eq!(
        eval(
            "Boolean",
            "ldc.i4 -1\nconv.i\nconv.u\nldc.i4 0\nconv.u\nclt"
        ),
        Value::Boolean(true)
    );
    assert_eq!(
        eval("Boolean", "ldc.i4 -1\nconv.i\nldc.i4 0\nconv.i\nclt.un"),
        Value::Boolean(false)
    );
    assert_eq!(
        eval("nint", "ldc.i4 -1\nconv.i\nldc.i4 2\nconv.i\ndiv.un"),
        Value::IntPtr(isize::MAX)
    );
    assert_eq!(
        eval("int32", "ldc.i4 -1\nldc.i4 2\ndiv.un"),
        Value::Int32(i32::MAX)
    );
}

#[test]
fn checked_native_arithmetic_faults_at_the_instruction() {
    let max = "ldc.i4 -1\nconv.i\nldc.i4 2\nconv.i\ndiv.un";
    for body in [
        format!("{max}\nldc.i4 1\nconv.i\nadd.ovf"),
        format!("{max}\nldc.i4 2\nconv.i\nmul.ovf"),
        format!("{max}\nldc.i4 1\nconv.i\nadd\nldc.i4 1\nconv.i\nsub.ovf"),
        format!("{max}\nldc.i4 1\nconv.i\nadd\nldc.i4 -1\nconv.i\ndiv"),
        "ldc.i4 -1\nconv.i\nldc.i4 1\nconv.i\nadd.ovf.un".into(),
        "ldc.i4 0\nconv.i\nldc.i4 1\nconv.i\nsub.ovf.un".into(),
        "ldc.i4 -1\nconv.i\nldc.i4 2\nconv.i\nmul.ovf.un".into(),
    ] {
        let fault = run(&module("nint", &body), Limits::default()).unwrap_err();
        assert!(fault.message.contains("overflow"), "{body}: {fault}");
        assert_eq!(fault.function.as_deref(), Some("Main"));
    }
    for op in ["div", "div.un"] {
        let fault = run(
            &module("nint", &format!("ldc.i4 1\nconv.i\nldc.i4 0\nconv.i\n{op}")),
            Limits::default(),
        )
        .unwrap_err();
        assert!(fault.message.contains("division by zero"));
    }
}

#[test]
fn native_integer_fields_round_trip_through_memory() {
    for (ty, conversion, expected) in [
        ("nint", "conv.i", Value::IntPtr(-1)),
        ("nuint", "conv.u", Value::UIntPtr(u32::MAX as usize)),
    ] {
        let body = format!(
            "ldc.i4 1\nheap.alloc {ty}\ndup\nldc.i4 -1\n{conversion}\nstobj {ty}\nldobj {ty}"
        );
        assert_eq!(eval(ty, &body), expected);
    }
    let source = ".module Test\n.entry Main\n.type Pair\n.field a nint\n.field b nuint\n.end\n.function Main() -> nuint\nldc.i4 1\nheap.alloc Pair\ndup\nldc.i4 -1\nconv.i\nldc.i4 42\nconv.u\nnewobj Pair\nstobj Pair\nldobj Pair\nldfld 1\nret\n.end";
    assert_eq!(
        run(&assemble(source).unwrap(), Limits::default())
            .unwrap()
            .value,
        Value::UIntPtr(42)
    );
}

#[test]
fn sample_round_trips_address_arithmetic_and_releases_storage() {
    let source = include_str!("../examples/native-integers.neoil");
    let loaded = load(&serde_json::to_string(&assemble(source).unwrap()).unwrap()).unwrap();
    let execution = run(&loaded, Limits::default()).unwrap();
    assert_eq!(execution.output, ["42"]);
    assert_eq!(execution.value, Value::Void);
    assert_eq!(execution.memory.live_allocations(), 0);
}

#[test]
fn native_integer_pointer_reconstruction_handles_null_and_unknown_addresses() {
    assert_eq!(
        eval(
            "Boolean",
            "ldc.i4 0\nconv.u\nptr.fromint int32\nptr.null int32\nceq"
        ),
        Value::Boolean(true)
    );
    assert_eq!(
        eval("nuint", "ldc.i4 1\nconv.u\nptr.fromint Void\nconv.u"),
        Value::UIntPtr(1)
    );
    let fault = run(
        &module("int32", "ldc.i4 1\nconv.u\nptr.fromint int32\nldind.i4"),
        Limits::default(),
    )
    .unwrap_err();
    assert!(fault.message.contains("untracked native pointer"));
    let fault = run(
        &module(
            "int32",
            "ldc.i4 1\nheap.alloc int32\ndup\nheap.free\npop\nconv.u\nptr.fromint int32\nldind.i4",
        ),
        Limits::default(),
    )
    .unwrap_err();
    assert!(fault.message.contains("untracked native pointer"));
}

#[test]
fn invalid_types_and_native_allocation_limits_fault() {
    for body in [
        "ldvoid\nconv.i",
        "ldc.i4 1\nptr.fromint int32",
        "ldc.i4 1\nconv.i\nldc.i4 1\nadd",
        "ldc.i4 1\nconv.i\nldc.i4 1\nconv.u\nadd",
        "ldc.i4 -1\nconv.i\nheap.alloc int32",
        "ldc.i4 -1\nconv.i\nconv.u\nheap.alloc int32",
    ] {
        assert!(
            run(&module("Void", body), Limits::default()).is_err(),
            "{body}"
        );
    }
    assert!(assemble(".module Test\n.function Unused() -> Void\nldc.i4 0\nconv.u\nptr.fromint Missing\npop\nldvoid\nret\n.end").is_err());
}
