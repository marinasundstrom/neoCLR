use neoclr::{Limits, Value, assemble, load, metadata::Type, run};

fn eval(ty: &str, body: &str) -> Value {
    let source = format!(".module Test\n.entry Main\n.function Main() -> {ty}\n{body}\nret\n.end");
    run(&assemble(&source).unwrap(), Limits::default())
        .unwrap()
        .value
}

#[test]
fn primitive_aliases_layouts_and_metadata_round_trip() {
    for (name, alias, ty, size) in [
        ("SByte", "int8", Type::SByte, 1),
        ("Byte", "uint8", Type::Byte, 1),
        ("Int16", "int16", Type::Int16, 2),
        ("UInt16", "uint16", Type::UInt16, 2),
        ("Char", "char", Type::Char, 2),
        ("UInt32", "uint32", Type::UInt32, 4),
        ("Int64", "int64", Type::Int64, 8),
        ("UInt64", "uint64", Type::UInt64, 8),
    ] {
        for spelling in [name.to_owned(), alias.to_owned(), format!("System.{name}")] {
            assert_eq!(Type::from_name(&spelling), ty);
            assert_eq!(
                eval("int32", &format!("sizeof {spelling}")),
                Value::Int32(size)
            );
        }
        assert!(
            neoclr::library::system()
                .unwrap()
                .type_definition(&ty)
                .is_some()
        );
        assert_eq!(
            serde_json::from_str::<Type>(&serde_json::to_string(&ty).unwrap()).unwrap(),
            ty
        );
    }
    let source = include_str!("../examples/integers.neoil");
    let loaded = load(&serde_json::to_string(&assemble(source).unwrap()).unwrap()).unwrap();
    let execution = run(&loaded, Limits::default()).unwrap();
    assert_eq!(execution.output, ["255", "-1"]);
    assert_eq!(execution.memory.live_allocations(), 0);
}

#[test]
fn small_integer_conversions_produce_int32_stack_values() {
    for (op, input, expected) in [
        ("conv.i1", 255, -1),
        ("conv.u1", -1, 255),
        ("conv.i2", 65535, -1),
        ("conv.u2", -1, 65535),
        ("conv.u4", -1, -1),
    ] {
        assert_eq!(
            eval("int32", &format!("ldc.i4 {input}\n{op}\nldc.i4 1\nadd")),
            Value::Int32(expected + 1)
        );
    }
    assert_eq!(eval("int64", "ldc.i4 -1\nconv.i8"), Value::Int64(-1));
    assert_eq!(
        eval("int64", "ldc.i4 -1\nconv.u8"),
        Value::Int64(u32::MAX as i64)
    );
    assert_eq!(eval("int32", "ldc.i8 4294967297\nconv.i4"), Value::Int32(1));
    assert_eq!(eval("int64", "ldc.i8 -1\nconv.u8"), Value::Int64(-1));
}

#[test]
fn locals_arguments_returns_and_fields_preserve_storage_width() {
    let source = ".module Test\n.entry Main\n.type Pair\n.field b Byte\n.field c Char\n.end\n.function Narrow(value: Byte) -> SByte\nldarg value\nret\n.end\n.function Main() -> int32\n.local pair: Pair\n.local small: Int16\nldc.i4 65535\nstloc small\nldloc small\ncall Narrow(Byte)\nldc.i4 55296\nnewobj Pair\nstloc pair\nldloc pair\nldfld 1\nldloc pair\nldfld 0\nadd\nret\n.end";
    assert_eq!(
        run(&assemble(source).unwrap(), Limits::default())
            .unwrap()
            .value,
        Value::Int32(55296 + 255)
    );
    assert_eq!(eval("Byte", "ldc.i4 511"), Value::Byte(255));
    assert_eq!(eval("Char", "ldc.i4 55296"), Value::Char(55296)); // UTF-16 surrogate is valid Char.
    assert_eq!(eval("UInt32", "ldc.i4 -1"), Value::UInt32(u32::MAX));
    assert_eq!(eval("UInt64", "ldc.i8 -1"), Value::UInt64(u64::MAX));
    assert_eq!(
        eval(
            "int32",
            ".local u: UInt32\nldc.i4 -1\nstloc u\nldloc u\nldc.i4 1\nadd"
        ),
        Value::Int32(0)
    );
}

#[test]
fn indirect_access_truncates_and_uses_opcode_signedness() {
    for (ty, store, load, value, expected) in [
        ("Byte", "stind.i1", "ldind.i1", 511, -1),
        ("SByte", "stind.i1", "ldind.u1", -1, 255),
        ("UInt16", "stind.i2", "ldind.i2", 131071, -1),
        ("Int16", "stind.i2", "ldind.u2", -1, 65535),
        ("Char", "stind.i2", "ldind.u2", 55296, 55296),
        ("UInt32", "stind.i4", "ldind.u4", -1, -1),
    ] {
        assert_eq!(
            eval(
                "int32",
                &format!("ldc.i4 1\nheap.alloc {ty}\ndup\nldc.i4 {value}\n{store}\n{load}")
            ),
            Value::Int32(expected)
        );
    }
    for ty in ["SByte", "Byte", "Int16", "UInt16", "Char", "UInt32"] {
        assert_eq!(
            eval(
                "int32",
                &format!("ldc.i4 1\nheap.alloc {ty}\ndup\nldc.i4 42\nstobj {ty}\nldobj {ty}")
            ),
            Value::Int32(42)
        );
    }
    assert_eq!(
        eval(
            "int64",
            "ldc.i4 1\nheap.alloc UInt64\ndup\nldc.i8 -1\nstind.i8\nldind.i8"
        ),
        Value::Int64(-1)
    );
}

#[test]
fn wide_arithmetic_and_unsigned_operations_preserve_all_bits() {
    assert_eq!(
        eval("int64", "ldc.i8 9223372036854775807\nldc.i8 1\nadd"),
        Value::Int64(i64::MIN)
    );
    assert_eq!(
        eval("int64", "ldc.i8 -1\nldc.i8 2\ndiv.un"),
        Value::Int64(i64::MAX)
    );
    assert_eq!(eval("int64", "ldc.i8 -7\nldc.i8 3\ndiv"), Value::Int64(-2));
    assert_eq!(
        eval("Boolean", "ldc.i8 -1\nldc.i8 0\nclt.un"),
        Value::Boolean(false)
    );
    assert_eq!(
        eval("int64", "ldc.i8 9007199254740993\nldc.i8 1\nsub"),
        Value::Int64(9007199254740992)
    );
    assert_eq!(
        eval("int64", "ldc.i8 3000000000\nldc.i8 3\nmul.ovf"),
        Value::Int64(9000000000)
    );
}

#[test]
fn wide_faults_and_invalid_memory_operations_report_locations() {
    for (body, expected) in [
        ("ldc.i8 9223372036854775807\nldc.i8 1\nadd.ovf", "overflow"),
        ("ldc.i8 -9223372036854775808\nldc.i8 -1\ndiv", "overflow"),
        ("ldc.i8 1\nldc.i8 0\ndiv.un", "division by zero"),
        ("ldc.i8 -1\nldc.i8 1\nadd.ovf.un", "overflow"),
        ("ldc.i4 1\nheap.alloc Byte\nldind.u1", "uninitialized"),
        ("ldc.i4 1\nheap.alloc Byte\nldind.i8", "type mismatch"),
        (
            "ldc.i4 2\nheap.alloc int64\nldc.i4 1\nptr.add\nldind.i8",
            "misaligned",
        ),
        (
            "ldc.i4 1\nheap.alloc Byte\nldc.i8 1\nstind.i1",
            "type mismatch",
        ),
        ("ldc.i4 1\nldc.i8 1\nadd", "matching integer types"),
    ] {
        let source =
            format!(".module Test\n.entry Main\n.function Main() -> Void\n{body}\nret\n.end");
        let fault = run(&assemble(&source).unwrap(), Limits::default()).unwrap_err();
        assert!(fault.message.contains(expected), "{fault}");
        assert_eq!(fault.function.as_deref(), Some("Main"));
        assert!(fault.instruction.is_some());
    }
    assert!(
        assemble(".module Test\n.function F() -> int64\nldc.i8 9223372036854775808\nret\n.end")
            .is_err()
    );
}
