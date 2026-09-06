use neoclr::{Limits, Value, assemble, load, metadata::Instruction, run};

fn program(body: &str) -> neoclr::Module {
    assemble(&format!(
        ".module Test\n.entry Main\n.function Main() -> Int32\n{body}\n.end"
    ))
    .unwrap()
}

#[test]
fn sample_roundtrips_and_exercises_switch_and_loop_exit() {
    let module = assemble(include_str!("../examples/control-flow.neoil")).unwrap();
    let loaded = load(&serde_json::to_string(&module).unwrap()).unwrap();
    assert_eq!(
        run(&loaded, Limits::default()).unwrap().output,
        ["zero", "one", "two", "outside table"]
    );
}

#[test]
fn switch_uses_zero_based_index_and_unsigned_fallthrough() {
    for (index, expected) in [
        (0, 10),
        (1, 20),
        (2, 20),
        (3, 99),
        (-1, 99),
        (i32::MIN, 99),
        (i32::MAX, 99),
    ] {
        let module = program(&format!(
            "ldc.i4 {index}\nswitch (First, Second, Second)\nldc.i4 99\nret\nFirst:\nldc.i4 10\nret\nSecond:\nldc.i4 20\nret"
        ));
        assert_eq!(
            run(&module, Limits::default()).unwrap().value,
            Value::Int32(expected)
        );
    }
}

#[test]
fn empty_switch_pops_index_and_preserves_older_stack_values() {
    let module = program("ldc.i4 42\nldc.i4 0\nswitch ()\nret");
    assert_eq!(
        run(&module, Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn switch_supports_backward_targets() {
    let module = program(
        ".local Int32 index\nldc.i4 0\nstloc index\nbr Test\nIncrement:\nldloc index\nldc.i4 1\nadd\nstloc index\nTest:\nldloc index\nswitch (Increment, Increment)\nldloc index\nret",
    );
    assert_eq!(
        run(&module, Limits::default()).unwrap().value,
        Value::Int32(2)
    );
}

#[test]
fn brfalse_consumes_condition_on_both_paths() {
    for (value, expected) in [(false, 42), (true, 99)] {
        let module = program(&format!(
            "ldc.bool {value}\nbrfalse Taken\nldc.i4 99\nret\nTaken:\nldc.i4 42\nret"
        ));
        assert_eq!(
            run(&module, Limits::default()).unwrap().value,
            Value::Int32(expected)
        );
    }
}

#[test]
fn malformed_tables_and_unresolved_labels_are_rejected() {
    for instruction in [
        "switch",
        "switch Label",
        "switch (Missing)",
        "switch (End,)",
        "switch (,End)",
        "switch (End End)",
        "brfalse Missing",
    ] {
        let result = assemble(&format!(
            ".module Test\n.function Main() -> Int32\nldc.i4 0\n{instruction}\nEnd:\nldc.i4 42\nret\n.end"
        ));
        assert!(result.is_err(), "{instruction}");
    }
}

#[test]
fn serialized_branch_targets_are_checked_even_when_unreachable() {
    for instruction in [
        Instruction::Switch(vec![usize::MAX]),
        Instruction::BranchFalse(usize::MAX),
    ] {
        let mut module = program("ldc.i4 42\nret");
        module.functions[0].body.push(instruction);
        assert!(load(&serde_json::to_string(&module).unwrap()).is_err());
    }
}

#[test]
fn operand_types_and_instruction_budget_are_checked() {
    for (body, message) in [
        (
            "ldc.bool true\nswitch ()\nldc.i4 42\nret",
            "switch requires Int32",
        ),
        (
            "ldc.i4 0\nbrfalse End\nEnd:\nldc.i4 42\nret",
            "brfalse requires Boolean",
        ),
        ("switch ()\nldc.i4 42\nret", "underflow"),
    ] {
        assert!(
            run(&program(body), Limits::default())
                .unwrap_err()
                .to_string()
                .contains(message)
        );
    }
    let module = program("Loop:\nldc.i4 0\nswitch (Loop)\nldc.i4 42\nret");
    assert!(
        run(
            &module,
            Limits {
                instructions: 10,
                ..Limits::default()
            }
        )
        .unwrap_err()
        .to_string()
        .contains("instruction limit")
    );
}
