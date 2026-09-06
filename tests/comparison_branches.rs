use neoclr::{Limits, Value, assemble, load, run};

const OPS: [&str; 10] = [
    "beq", "bne.un", "bgt", "bgt.un", "blt", "blt.un", "bge", "bge.un", "ble", "ble.un",
];

fn branch(op: &str, operands: &str) -> bool {
    let module = assemble(&format!(".module Test\n.entry Main\n.function Main() -> Int32\nldc.i4 40\n{operands}\n{op} Taken\nldc.i4 1\nadd\nret\nTaken:\nldc.i4 2\nadd\nret\n.end")).unwrap();
    let value = run(&module, Limits::default()).unwrap().value;
    assert!(matches!(value, Value::Int32(41 | 42)));
    value == Value::Int32(42)
}

#[test]
fn integer_branches_distinguish_signed_unsigned_and_equality_boundaries() {
    for (a, b) in [
        (-1i32, 0i32),
        (0, -1),
        (0, 0),
        (i32::MIN, i32::MAX),
        (7, 8),
        (8, 7),
    ] {
        for op in OPS {
            let expected = match op {
                "beq" => a == b,
                "bne.un" => a != b,
                "bgt" => a > b,
                "bgt.un" => (a as u32) > (b as u32),
                "blt" => a < b,
                "blt.un" => (a as u32) < (b as u32),
                "bge" => a >= b,
                "bge.un" => (a as u32) >= (b as u32),
                "ble" => a <= b,
                "ble.un" => (a as u32) <= (b as u32),
                _ => unreachable!(),
            };
            assert_eq!(
                branch(op, &format!("ldc.i4 {a}\nldc.i4 {b}")),
                expected,
                "{a} {op} {b}"
            );
        }
    }
}

#[test]
fn floating_branches_handle_nan_in_either_position_and_signed_zero() {
    for (a, b) in [
        (f64::NAN, 0.0),
        (0.0, f64::NAN),
        (f64::NAN, f64::NAN),
        (-0.0, 0.0),
        (f64::INFINITY, 1.0),
        (-2.0, 1.0),
        (1.0, -2.0),
    ] {
        for op in OPS {
            let unordered = a.is_nan() || b.is_nan();
            let expected = match op {
                "beq" => a == b,
                "bne.un" => a != b,
                "bgt" => a > b,
                "bgt.un" => unordered || a > b,
                "blt" => a < b,
                "blt.un" => unordered || a < b,
                "bge" => a >= b,
                "bge.un" => unordered || a >= b,
                "ble" => a <= b,
                "ble.un" => unordered || a <= b,
                _ => unreachable!(),
            };
            assert_eq!(
                branch(op, &format!("ldc.r8 {a}\nldc.r8 {b}")),
                expected,
                "{a} {op} {b}"
            );
        }
    }
}

#[test]
fn wide_and_native_integer_branches_preserve_width_and_opcode_signedness() {
    for operands in [
        "ldc.i8 -1\nldc.i8 0",
        "ldc.i4 -1\nconv.i\nldc.i4 0\nconv.i",
        "ldc.i8 -1\nconv.u\nldc.i4 0\nconv.u",
    ] {
        for (op, expected) in [
            ("blt", true),
            ("bge", false),
            ("bgt.un", true),
            ("ble.un", false),
            ("beq", false),
            ("bne.un", true),
        ] {
            assert_eq!(branch(op, operands), expected, "{op}: {operands}");
        }
    }
}

#[test]
fn equality_uses_existing_value_and_pointer_equality() {
    assert!(branch("beq", "ptr.null Int32\nptr.null Int32"));
    assert!(branch("beq", "ldc.i4 1\nconv.u\nptr.fromint Int32\ndup"));
    assert!(branch("beq", "ldstr \"same\"\nldstr \"same\""));
    assert!(branch("beq", "ldvoid\nldvoid"));
    assert!(branch("bne.un", "ldc.i4 1\nheap.new\nldc.i4 1\nheap.new"));
}

#[test]
fn every_opcode_validates_labels_serialized_targets_and_operand_types() {
    for op in OPS {
        assert!(
            assemble(&format!(
                ".module Test\n.function F() -> Void\n{op} Missing\nldvoid\nret\n.end"
            ))
            .is_err()
        );
        let module = assemble(&format!(".module Test\n.entry Main\n.function Main() -> Void\nldc.i4 1\nldc.i8 1\n{op} End\nEnd:\nldvoid\nret\n.end")).unwrap();
        assert!(run(&module, Limits::default()).is_err());
        let mut json = serde_json::to_value(module).unwrap();
        json["functions"][0]["body"][2]["arg"] = serde_json::json!(99999);
        assert!(load(&json.to_string()).is_err());
    }
}

#[test]
fn backward_comparison_branch_obeys_instruction_budget() {
    let module = assemble(".module Test\n.entry Main\n.function Main() -> Void\nLoop:\nldc.i4 1\nldc.i4 1\nbeq Loop\nldvoid\nret\n.end").unwrap();
    assert!(
        run(
            &module,
            Limits {
                instructions: 12,
                ..Limits::default()
            }
        )
        .unwrap_err()
        .message
        .contains("instruction limit")
    );
}

#[test]
fn sample_roundtrips_and_runs() {
    let module = assemble(include_str!("../examples/comparison-branches.neoil")).unwrap();
    let module = load(&serde_json::to_string(&module).unwrap()).unwrap();
    assert_eq!(
        run(&module, Limits::default()).unwrap().output,
        ["Comparison branches passed"]
    );
}
