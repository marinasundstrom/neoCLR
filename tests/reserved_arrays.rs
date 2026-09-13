use neoclr::{Limits, Value, assemble, run, verify};
fn program(body: &str) -> neoclr::Module {
    assemble(&format!(
        ".module Reserved\n.entry Main\n.function Main() -> Int32\n{body}\n.end"
    ))
    .unwrap()
}
#[test]
fn reserved_slots_are_not_default_values_and_must_be_written() {
    for load in ["ldelem Int32", "ldelema Int32\nldobj Int32"] {
        let m = program(&format!(
            "ldc.i4 1\narray.reserve Int32\nldc.i4 0\n{load}\nret"
        ));
        verify(&m).unwrap();
        assert!(
            run(&m, Limits::default())
                .unwrap_err()
                .message
                .contains("uninitialized")
        );
    }
    let m = program(
        ".local arrayref<Int32> values\nldc.i4 1\narray.reserve Int32\nstloc values\nldloc values\nldc.i4 0\nldc.i4 42\nstelem Int32\nldloc values\nldc.i4 0\nldelem Int32\nret",
    );
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(42));
}
#[test]
fn ordinary_newarr_keeps_its_zero_default() {
    let m = program("ldc.i4 1\nnewarr Int32\nldc.i4 0\nldelem Int32\nret");
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(0));
}
#[test]
fn capacity_can_hold_a_union_without_inventing_a_default_case() {
    let m = program(
        r#"
.local arrayref<System.Result<Int32,System.OverflowError>> values
ldc.i4 2
array.reserve System.Result<Int32,System.OverflowError>
stloc values
ldloc values
ldc.i4 0
ldc.i4 42
call System.Result<Int32,System.OverflowError>::Ok(Int32)
stelem System.Result<Int32,System.OverflowError>
ldloc values
ldc.i4 0
ldelem System.Result<Int32,System.OverflowError>
call instance System.Result<Int32,System.OverflowError>::GetOkCase()
call instance System.Result.Ok<Int32>::get_Value()
ret
"#,
    );
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(42));
}
