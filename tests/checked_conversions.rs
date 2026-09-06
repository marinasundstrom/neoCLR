use neoclr::{Limits, Value, assemble, load, run};

fn execute(returns: &str, body: &str) -> Result<neoclr::Execution, neoclr::Fault> {
    let source =
        format!(".module Test\n.entry Main\n.function Main() -> {returns}\n{body}\nret\n.end");
    run(&assemble(&source).unwrap(), Limits::default())
}
fn eval(returns: &str, body: &str) -> Value {
    execute(returns, body).unwrap().value
}
fn overflow(returns: &str, body: &str) {
    let fault = execute(returns, body).unwrap_err();
    assert!(
        fault.message.contains("checked conversion overflow"),
        "{body}: {fault}"
    );
    assert_eq!(fault.function.as_deref(), Some("Main"));
    assert!(fault.instruction.is_some());
}

#[test]
fn every_checked_opcode_executes_and_round_trips_in_sample() {
    for (suffix, returns, expected) in [
        ("i1", "Int32", Value::Int32(42)),
        ("u1", "Int32", Value::Int32(42)),
        ("i2", "Int32", Value::Int32(42)),
        ("u2", "Int32", Value::Int32(42)),
        ("i4", "Int32", Value::Int32(42)),
        ("u4", "Int32", Value::Int32(42)),
        ("i8", "Int64", Value::Int64(42)),
        ("u8", "Int64", Value::Int64(42)),
        ("i", "IntPtr", Value::IntPtr(42)),
        ("u", "UIntPtr", Value::UIntPtr(42)),
    ] {
        for unsigned in ["", ".un"] {
            assert_eq!(
                eval(returns, &format!("ldc.i8 42\nconv.ovf.{suffix}{unsigned}")),
                expected
            );
        }
    }
    let sample = assemble(include_str!("../examples/checked-conversions.neoil")).unwrap();
    let loaded = load(&serde_json::to_string(&sample).unwrap()).unwrap();
    let execution = run(&loaded, Limits::default()).unwrap();
    assert_eq!(execution.output, ["42", "255"]);
    assert_eq!(execution.value, Value::Void);
}

#[test]
fn fixed_width_integer_range_boundaries_are_exact() {
    for (suffix, lower, upper) in [
        ("i1", -128i64, 127i64),
        ("u1", 0, 255),
        ("i2", -32768, 32767),
        ("u2", 0, 65535),
        ("i4", i32::MIN as i64, i32::MAX as i64),
        ("u4", 0, u32::MAX as i64),
    ] {
        for number in [lower, upper] {
            assert_eq!(
                eval("Int32", &format!("ldc.i8 {number}\nconv.ovf.{suffix}")),
                Value::Int32(number as i32)
            );
        }
        for number in [lower - 1, upper + 1] {
            overflow("Int32", &format!("ldc.i8 {number}\nconv.ovf.{suffix}"));
        }
        assert_eq!(
            eval("Int32", &format!("ldc.i8 {upper}\nconv.ovf.{suffix}.un")),
            Value::Int32(upper as i32)
        );
        overflow("Int32", &format!("ldc.i8 -1\nconv.ovf.{suffix}.un"));
    }
}

#[test]
fn source_unsignedness_is_independent_of_destination() {
    overflow("Int64", "ldc.i8 -1\nconv.ovf.u8");
    overflow("Int64", "ldc.i8 -1\nconv.ovf.i8.un");
    assert_eq!(eval("Int64", "ldc.i8 -1\nconv.ovf.u8.un"), Value::Int64(-1));
    assert_eq!(
        eval("Int64", "ldc.i4 -1\nconv.ovf.i8.un"),
        Value::Int64(u32::MAX as i64)
    );
    assert_eq!(eval("Int64", "ldc.i4 -1\nconv.ovf.i8"), Value::Int64(-1));
    for number in [i64::MIN, i64::MAX, 9007199254740993] {
        assert_eq!(
            eval("Int64", &format!("ldc.i8 {number}\nconv.ovf.i8")),
            Value::Int64(number)
        );
    }
    // A normalized UInt32 local must be explicitly interpreted as unsigned.
    assert_eq!(
        eval(
            "Int64",
            ".local u: UInt32\nldc.i4 -1\nstloc u\nldloc u\nconv.ovf.u8.un"
        ),
        Value::Int64(u32::MAX as i64)
    );
}

#[test]
fn native_destinations_follow_host_width_without_precision_loss() {
    assert_eq!(
        eval("IntPtr", &format!("ldc.i8 {}\nconv.ovf.i", isize::MAX)),
        Value::IntPtr(isize::MAX)
    );
    assert_eq!(
        eval("IntPtr", &format!("ldc.i8 {}\nconv.ovf.i", isize::MIN)),
        Value::IntPtr(isize::MIN)
    );
    assert_eq!(
        eval("UIntPtr", "ldc.i4 0\nconv.u\nnot\nconv.ovf.u.un"),
        Value::UIntPtr(usize::MAX)
    );
    overflow("UIntPtr", "ldc.i4 0\nconv.u\nnot\nconv.ovf.u");
    overflow("IntPtr", "ldc.i4 0\nconv.u\nnot\nconv.ovf.i.un");
    let result = execute("IntPtr", "ldc.i8 2147483648\nconv.ovf.i");
    if usize::BITS == 32 {
        assert!(result.is_err());
    } else {
        assert_eq!(result.unwrap().value, Value::IntPtr(2147483648i64 as isize));
    }
}

#[test]
fn floating_conversion_truncates_then_checks_exact_exclusive_bounds() {
    for suffix in ["", ".un"] {
        assert_eq!(
            eval("Int32", &format!("ldc.r8 -128.9\nconv.ovf.i1{suffix}")),
            Value::Int32(-128)
        );
        assert_eq!(
            eval("Int32", &format!("ldc.r8 255.9\nconv.ovf.u1{suffix}")),
            Value::Int32(255)
        );
        assert_eq!(
            eval("Int32", &format!("ldc.r8 -0.9\nconv.ovf.u1{suffix}")),
            Value::Int32(0)
        );
        overflow("Int32", &format!("ldc.r8 256\nconv.ovf.u1{suffix}"));
    }
    for (suffix, returns, exponent) in [
        ("i8", "Int64", 63),
        ("u8", "Int64", 64),
        ("i", "IntPtr", usize::BITS - 1),
        ("u", "UIntPtr", usize::BITS),
    ] {
        let bound = 2f64.powi(exponent as i32);
        let below = f64::from_bits(bound.to_bits() - 1);
        overflow(returns, &format!("ldc.r8 {bound:.0}\nconv.ovf.{suffix}"));
        assert!(execute(returns, &format!("ldc.r8 {below:.0}\nconv.ovf.{suffix}")).is_ok());
    }
    assert_eq!(
        eval("Int64", "ldc.r8 -9223372036854775808\nconv.ovf.i8"),
        Value::Int64(i64::MIN)
    );
    overflow("Int64", "ldc.r8 -9223372036854777856\nconv.ovf.i8");
    for literal in ["NaN", "inf", "-inf"] {
        for suffix in ["i1", "u1", "i2", "u2", "i4", "u4", "i8", "u8", "i", "u"] {
            overflow("Void", &format!("ldc.r8 {literal}\nconv.ovf.{suffix}"));
            overflow("Void", &format!("ldc.r8 {literal}\nconv.ovf.{suffix}.un"));
        }
    }
}

#[test]
fn nonnumeric_values_and_malformed_checked_instructions_are_rejected() {
    for body in [
        "ldvoid\nconv.ovf.i4",
        "ldc.bool true\nconv.ovf.i4",
        "ptr.null Int32\nconv.ovf.u",
    ] {
        let fault = execute("Void", body).unwrap_err();
        assert!(fault.message.contains("requires numeric value"));
        assert_eq!(fault.function.as_deref(), Some("Main"));
    }
    assert!(
        execute("Void", "conv.ovf.i4")
            .unwrap_err()
            .message
            .contains("underflow")
    );
    for instruction in ["conv.ovf.r4", "conv.ovf.i4 42", "conv.ovf.i1.un.un"] {
        assert!(
            assemble(&format!(
                ".module Test\n.function F() -> Void\n{instruction}\nret\n.end"
            ))
            .is_err()
        );
    }
}
