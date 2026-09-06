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
fn number(body: &str) -> f64 {
    let Value::Double(value) = eval("Double", body) else {
        panic!("expected Double")
    };
    value
}

#[test]
fn primitive_identity_layout_and_sample() {
    for (aliases, ty, size) in [
        (["Single", "float32", "System.Single"], Type::Single, 4),
        (["Double", "float64", "System.Double"], Type::Double, 8),
    ] {
        for alias in aliases {
            assert_eq!(Type::from_name(alias), ty);
            assert_eq!(
                eval("int32", &format!("sizeof {alias}")),
                Value::Int32(size)
            );
        }
        assert!(
            neoclr::library::system()
                .unwrap()
                .type_definition(&ty)
                .is_some()
        );
    }
    let module = assemble(include_str!("../examples/floating.neoil")).unwrap();
    let loaded = load(&serde_json::to_string(&module).unwrap()).unwrap();
    let execution = run(&loaded, Limits::default()).unwrap();
    assert_eq!(execution.output, ["42"]);
    assert_eq!(execution.value, Value::Void);
    assert_eq!(execution.memory.live_allocations(), 0);
}

#[test]
fn metadata_serializes_literal_bits_without_json_float_loss() {
    for literal in ["-0.0", "NaN", "inf", "-inf", "5e-324"] {
        for op in ["ldc.r4", "ldc.r8"] {
            let original = module("Double", &format!("{op} {literal}"));
            let json = serde_json::to_string(&original).unwrap();
            assert!(json.contains("bits"));
            let encoded: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert!(encoded["functions"][0]["body"][0]["arg"]["bits"].is_u64());
            let loaded = load(&json).unwrap();
            assert_eq!(serde_json::to_string(&loaded).unwrap(), json);
            let Value::Double(a) = run(&original, Limits::default()).unwrap().value else {
                panic!()
            };
            let Value::Double(b) = run(&loaded, Limits::default()).unwrap().value else {
                panic!()
            };
            assert_eq!(a.to_bits(), b.to_bits());
        }
    }
    assert!(assemble(".module Test\n.function F() -> Double\nldc.r8 nope\nret\n.end").is_err());
}

#[test]
fn arithmetic_uses_ieee_nonfinite_values_and_signed_zero() {
    for (op, expected) in [
        ("add", 9.5),
        ("sub", 5.5),
        ("mul", 15.0),
        ("div", 3.75),
        ("rem", 1.5),
    ] {
        assert_eq!(number(&format!("ldc.r8 7.5\nldc.r8 2\n{op}")), expected);
    }
    assert!(number("ldc.r8 0\nldc.r8 0\ndiv").is_nan());
    assert_eq!(number("ldc.r8 1\nldc.r8 -0.0\ndiv"), f64::NEG_INFINITY);
    assert!(number("ldc.r8 inf\nldc.r8 2\nrem").is_nan());
    assert_eq!(number("ldc.r8 -7.5\nldc.r8 2\nrem"), -1.5);
    assert_eq!(number("ldc.r8 3\nldc.r8 inf\nrem"), 3.0);
    assert!(number("ldc.r8 1e308\nldc.r8 2\nmul").is_infinite());
    assert_eq!(number("ldc.r8 0.0\nneg").to_bits(), (-0.0f64).to_bits());
    assert_eq!(
        number("ldc.r8 -4\nldc.r8 2\nrem").to_bits(),
        (-0.0f64).to_bits()
    );
}

#[test]
fn comparisons_handle_unordered_nan_and_equal_zero_signs() {
    assert_eq!(
        eval("Boolean", "ldc.i4 -1\nldc.i4 0\ncgt"),
        Value::Boolean(false)
    );
    assert_eq!(
        eval("Boolean", "ldc.i8 -1\nldc.i8 0\ncgt.un"),
        Value::Boolean(true)
    );
    for op in ["ceq", "clt", "cgt"] {
        assert_eq!(
            eval("Boolean", &format!("ldc.r8 NaN\nldc.r8 NaN\n{op}")),
            Value::Boolean(false)
        );
    }
    for pair in ["ldc.r8 NaN\nldc.r8 1", "ldc.r8 1\nldc.r8 NaN"] {
        for op in ["clt.un", "cgt.un"] {
            assert_eq!(
                eval("Boolean", &format!("{pair}\n{op}")),
                Value::Boolean(true)
            );
        }
    }
    assert_eq!(
        eval("Boolean", "ldc.r8 -0\nldc.r8 0\nceq"),
        Value::Boolean(true)
    );
    assert_eq!(
        eval("Boolean", "ldc.r4 1.5\nldc.r8 2.5\nclt"),
        Value::Boolean(true)
    );
}

#[test]
fn single_storage_and_explicit_conversion_round_but_stack_keeps_binary64() {
    assert_eq!(number("ldc.r8 16777217\nconv.r4"), 16777216.0);
    assert_eq!(number("ldc.r4 16777216\nldc.r4 1\nadd"), 16777217.0);
    assert_eq!(
        number(".local s: Single\nldc.r8 16777217\nstloc s\nldloc s"),
        16777216.0
    );
    assert_eq!(eval("Single", "ldc.r8 16777217"), Value::Single(16777216.0));
    let source = ".module Test\n.entry Main\n.type Box\n.field s Single\n.end\n.function Round(s: Single) -> Single\nldarg s\nret\n.end\n.function Main() -> Double\nldc.r8 16777217\ncall Round(Single)\nnewobj Box\nldfld 0\nret\n.end";
    assert_eq!(
        run(&assemble(source).unwrap(), Limits::default())
            .unwrap()
            .value,
        Value::Double(16777216.0)
    );
    // Just above an f32 midpoint: going through f64 first would round incorrectly.
    assert_eq!(
        number("ldc.i8 4611686293305294849\nconv.r4"),
        f32::from_bits(0x5e800001) as f64
    );
}

#[test]
fn floating_native_storage_supports_typed_and_indirect_access() {
    for (ty, store, read) in [
        ("Single", "stind.r4", "ldind.r4"),
        ("Double", "stind.r8", "ldind.r8"),
        ("Single", "stobj Single", "ldobj Single"),
        ("Double", "stobj Double", "ldobj Double"),
    ] {
        let result = number(&format!(
            "ldc.i4 1\nheap.alloc {ty}\ndup\nldc.r8 -0.0\n{store}\n{read}"
        ));
        assert_eq!(result.to_bits(), (-0.0f64).to_bits());
    }
    // A pointer cast can reinterpret the actual IEEE representation.
    assert_eq!(
        eval(
            "int32",
            "ldc.i4 1\nheap.alloc Single\ndup\nldc.r4 1.0\nstind.r4\nptr.cast int32\nldind.i4"
        ),
        Value::Int32(0x3f800000)
    );
}

#[test]
fn conversions_distinguish_signedness_truncation_and_unspecified_range_policy() {
    assert_eq!(number("ldc.i4 -1\nconv.r8"), -1.0);
    assert_eq!(number("ldc.i4 -1\nconv.r.un"), u32::MAX as f64);
    assert_eq!(number("ldc.i8 -1\nconv.r.un"), u64::MAX as f64);
    assert_eq!(eval("int32", "ldc.r8 -3.9\nconv.i4"), Value::Int32(-3));
    assert_eq!(
        eval("int64", "ldc.r8 1e100\nconv.i8"),
        Value::Int64(i64::MAX)
    );
    assert_eq!(eval("int32", "ldc.r8 NaN\nconv.i4"), Value::Int32(0));
    assert_eq!(eval("int32", "ldc.r8 -inf\nconv.u4"), Value::Int32(0));
    assert_eq!(eval("int32", "ldc.r8 inf\nconv.u1"), Value::Int32(255));
    assert_eq!(eval("nint", "ldc.r8 42.9\nconv.i"), Value::IntPtr(42));
    assert_eq!(eval("nuint", "ldc.r8 42.9\nconv.u"), Value::UIntPtr(42));
}

#[test]
fn finite_checks_and_invalid_float_instructions_fault_with_locations() {
    assert_eq!(number("ldc.r8 1.25\nckfinite"), 1.25);
    for (body, message) in [
        ("ldc.r8 NaN\nckfinite", "non-finite"),
        ("ldc.r8 inf\nckfinite", "non-finite"),
        ("ldc.i4 1\nckfinite", "requires floating-point"),
        (
            "ldc.r8 1\nldc.r8 2\nadd.ovf",
            "does not accept floating-point",
        ),
        (
            "ldc.r8 1\nldc.r8 2\ndiv.un",
            "does not accept floating-point",
        ),
        (
            "ldc.r8 1\nconv.r.un",
            "unsupported floating-point conversion",
        ),
        ("ldc.r8 1\nldc.i4 2\nadd", "matching integer types"),
        ("ldc.i4 1\nheap.alloc Single\nldind.r4", "uninitialized"),
        ("ldc.i4 1\nheap.alloc Single\nldind.r8", "type mismatch"),
        (
            "ldc.i4 1\nheap.alloc Single\nldc.i4 1\nstind.r4",
            "type mismatch",
        ),
    ] {
        let fault = run(&module("Void", body), Limits::default()).unwrap_err();
        assert!(fault.message.contains(message), "{body}: {fault}");
        assert_eq!(fault.function.as_deref(), Some("Main"));
        assert!(fault.instruction.is_some());
    }
}
