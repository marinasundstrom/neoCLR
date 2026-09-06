use neoclr::{Limits, Value, assemble, load, run};

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
fn sample_round_trips_and_demonstrates_bit_fields_and_signedness() {
    let module = assemble(include_str!("../examples/bits.neoil")).unwrap();
    let module = load(&serde_json::to_string(&module).unwrap()).unwrap();
    let execution = run(&module, Limits::default()).unwrap();
    assert_eq!(execution.output, ["18", "-1", "5", "-4"]);
    assert_eq!(execution.value, Value::Void);
}

#[test]
fn bitwise_operations_cover_all_integer_stack_categories() {
    for (op, answer) in [("and", 8), ("or", 14), ("xor", 6)] {
        assert_eq!(
            eval("int32", &format!("ldc.i4 12\nldc.i4 10\n{op}")),
            Value::Int32(answer)
        );
        assert_eq!(
            eval("int64", &format!("ldc.i8 12\nldc.i8 10\n{op}")),
            Value::Int64(answer as i64)
        );
        assert_eq!(
            eval(
                "nint",
                &format!("ldc.i4 12\nconv.i\nldc.i4 10\nconv.i\n{op}")
            ),
            Value::IntPtr(answer as isize)
        );
        assert_eq!(
            eval(
                "nuint",
                &format!("ldc.i4 12\nconv.u\nldc.i4 10\nconv.u\n{op}")
            ),
            Value::UIntPtr(answer as usize)
        );
    }
    assert_eq!(
        eval("int64", "ldc.i8 4294967296\nldc.i8 1\nor"),
        Value::Int64(4294967297)
    );
    assert_eq!(eval("int32", "ldc.i4 0\nnot"), Value::Int32(-1));
    assert_eq!(eval("int64", "ldc.i8 0\nnot"), Value::Int64(-1));
    assert_eq!(eval("nint", "ldc.i4 0\nconv.i\nnot"), Value::IntPtr(-1));
    assert_eq!(
        eval("nuint", "ldc.i4 0\nconv.u\nnot"),
        Value::UIntPtr(usize::MAX)
    );
}

#[test]
fn negation_wraps_at_signed_minimum_and_native_unsigned_width() {
    assert_eq!(
        eval("int32", "ldc.i4 -2147483648\nneg"),
        Value::Int32(i32::MIN)
    );
    assert_eq!(
        eval("int64", "ldc.i8 -9223372036854775808\nneg"),
        Value::Int64(i64::MIN)
    );
    assert_eq!(
        eval("nint", &format!("ldc.i8 {}\nconv.i\nneg", isize::MIN)),
        Value::IntPtr(isize::MIN)
    );
    assert_eq!(
        eval("nuint", "ldc.i4 1\nconv.u\nneg"),
        Value::UIntPtr(usize::MAX)
    );
    assert_eq!(eval("int32", "ldc.i4 -42\nneg"), Value::Int32(42));
}

#[test]
fn shifts_use_opcode_signedness_and_mask_counts_at_the_value_width() {
    assert_eq!(eval("int32", "ldc.i4 -8\nldc.i4 1\nshr"), Value::Int32(-4));
    assert_eq!(
        eval("int32", "ldc.i4 -1\nldc.i4 1\nshr.un"),
        Value::Int32(i32::MAX)
    );
    assert_eq!(
        eval("int64", "ldc.i8 -1\nldc.i4 1\nshr.un"),
        Value::Int64(i64::MAX)
    );
    assert_eq!(
        eval("int64", "ldc.i8 1\nldc.i4 40\nshl"),
        Value::Int64(1 << 40)
    );
    for count in [0, 32, 64] {
        assert_eq!(
            eval("int32", &format!("ldc.i4 1\nldc.i4 {count}\nshl")),
            Value::Int32(1)
        );
    }
    assert_eq!(
        eval("int32", "ldc.i4 1\nldc.i4 -1\nshl"),
        Value::Int32(i32::MIN)
    );
    assert_eq!(
        eval("int64", "ldc.i8 1\nldc.i4 -1\nshl"),
        Value::Int64(i64::MIN)
    );
    assert_eq!(eval("int64", "ldc.i8 -8\nldc.i4 65\nshr"), Value::Int64(-4));
    assert_eq!(
        eval(
            "nint",
            &format!("ldc.i4 1\nconv.i\nldc.i4 {}\nshl", usize::BITS)
        ),
        Value::IntPtr(1)
    );
    assert_eq!(
        eval("nuint", "ldc.i4 0\nconv.u\nnot\nldc.i4 1\nshr"),
        Value::UIntPtr(usize::MAX)
    );
    assert_eq!(
        eval("nuint", "ldc.i4 0\nconv.u\nnot\nldc.i4 1\nshr.un"),
        Value::UIntPtr(isize::MAX as usize)
    );
    for conversion in ["conv.i", "conv.u"] {
        assert_eq!(
            eval("int32", &format!("ldc.i4 1\nldc.i4 2\n{conversion}\nshl")),
            Value::Int32(4)
        );
    }
}

#[test]
fn remainder_follows_dividend_sign_and_unsigned_bit_interpretation() {
    for (left, right, answer) in [(-7, 3, -1), (7, -3, 1), (-7, -3, -1), (6, 3, 0)] {
        assert_eq!(
            eval("int32", &format!("ldc.i4 {left}\nldc.i4 {right}\nrem")),
            Value::Int32(answer)
        );
        assert_eq!(
            eval("int64", &format!("ldc.i8 {left}\nldc.i8 {right}\nrem")),
            Value::Int64(answer as i64)
        );
    }
    assert_eq!(
        eval("int32", "ldc.i4 -1\nldc.i4 10\nrem.un"),
        Value::Int32(5)
    );
    assert_eq!(
        eval("int64", "ldc.i8 -1\nldc.i8 10\nrem.un"),
        Value::Int64(5)
    );
    assert_eq!(
        eval("nint", "ldc.i4 -7\nconv.i\nldc.i4 3\nconv.i\nrem"),
        Value::IntPtr(-1)
    );
    assert_eq!(
        eval("nuint", "ldc.i4 0\nconv.u\nnot\nldc.i4 10\nconv.u\nrem.un"),
        Value::UIntPtr(usize::MAX % 10)
    );
}

#[test]
fn invalid_operands_and_remainder_failures_fault_with_locations() {
    for (body, message) in [
        ("ldc.i4 1\nldc.i4 0\nrem", "division by zero"),
        ("ldc.i8 1\nldc.i8 0\nrem.un", "division by zero"),
        ("ldc.i4 -2147483648\nldc.i4 -1\nrem", "overflow"),
        ("ldc.i8 -9223372036854775808\nldc.i8 -1\nrem", "overflow"),
        ("ldvoid\nnot", "requires integer"),
        ("ldc.bool true\nneg", "requires integer"),
        ("ldc.i4 1\nldc.i8 1\nand", "matching integer types"),
        ("ldc.i4 1\nldc.i8 1\nshl", "shift count"),
        ("ldvoid\nldc.i4 1\nshr", "shift requires integer"),
        ("shl", "underflow"),
    ] {
        let fault = run(&module("Void", body), Limits::default()).unwrap_err();
        assert!(fault.message.contains(message), "{body}: {fault}");
        assert_eq!(fault.function.as_deref(), Some("Main"));
        assert!(fault.instruction.is_some());
    }
}
