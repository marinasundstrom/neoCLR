use neoclr::{Limits, LoadedProgram, Value, assemble, assembler::parse_function_ref};

#[test]
fn division_result_preserves_quotients_and_distinguishes_error_cases() {
    let module = assemble(".module App").unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    let resolve = |name: &str| {
        program
            .resolve_function(&parse_function_ref(name).unwrap())
            .unwrap()
    };
    let divide = resolve("System.Int32::Divide(Int32,Int32)");
    for (a, b, expected) in [
        (7, 2, 3),
        (-7, 2, -3),
        (7, -2, -3),
        (-7, -2, 3),
        (i32::MIN, 1, i32::MIN),
        (i32::MAX, 1, i32::MAX),
        (0, -1, 0),
    ] {
        let result = divide
            .invoke(vec![Value::Int32(a), Value::Int32(b)], Limits::default())
            .unwrap()
            .value;
        let case =
            resolve("instance System.Result<Int32,System.IntegerDivisionError>::GetOkCase()")
                .invoke_instance(result, vec![], Limits::default())
                .unwrap()
                .value;
        assert_eq!(
            resolve("instance System.Result.Ok<Int32>::get_Value()")
                .invoke_instance(case, vec![], Limits::default())
                .unwrap()
                .value,
            Value::Int32(expected)
        );
    }
    for (a, b, name) in [
        (0, 0, "DivisionByZero"),
        (i32::MIN, 0, "DivisionByZero"),
        (1, 0, "DivisionByZero"),
        (i32::MIN, -1, "Overflow"),
    ] {
        let result = divide
            .invoke(vec![Value::Int32(a), Value::Int32(b)], Limits::default())
            .unwrap()
            .value;
        let case =
            resolve("instance System.Result<Int32,System.IntegerDivisionError>::GetErrorCase()")
                .invoke_instance(result, vec![], Limits::default())
                .unwrap()
                .value;
        let error =
            resolve("instance System.Result.Error<System.IntegerDivisionError>::get_Value()")
                .invoke_instance(case, vec![], Limits::default())
                .unwrap()
                .value;
        assert_eq!(
            resolve(&format!(
                "instance System.IntegerDivisionError::get_Is{name}()"
            ))
            .invoke_instance(error.clone(), vec![], Limits::default())
            .unwrap()
            .value,
            Value::Boolean(true)
        );
        let extracted = resolve(&format!(
            "instance System.IntegerDivisionError::Get{name}()"
        ))
        .invoke_instance(error.clone(), vec![], Limits::default())
        .unwrap()
        .value;
        assert_eq!(
            extracted.ty(),
            neoclr::assembler::parse_type(&format!("System.IntegerDivisionError.{name}")).unwrap()
        );
        let other = if name == "Overflow" {
            "DivisionByZero"
        } else {
            "Overflow"
        };
        assert!(
            resolve(&format!(
                "instance System.IntegerDivisionError::Get{other}()"
            ))
            .invoke_instance(error.clone(), vec![], Limits::default())
            .is_err()
        );
        assert_eq!(
            resolve("instance System.IntegerDivisionError::ToString()")
                .invoke_instance(error, vec![], Limits::default())
                .unwrap()
                .value,
            Value::String(name.into())
        );
    }
}

#[test]
fn abs_overflow_is_an_ordinary_error_value_and_old_result_type_is_rejected() {
    let source = ".module App\n.entry Main\n.function Main() -> System.OverflowError\nldc.i4 -2147483648\ncall System.Math::Abs(Int32)\ncall instance System.Result<Int32,System.OverflowError>::GetErrorCase()\ncall instance System.Result.Error<System.OverflowError>::get_Value()\nret\n.end";
    let module = assemble(source).unwrap();
    let loaded = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    let program = LoadedProgram::new(&loaded).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Object {
            ty: neoclr::assembler::parse_type("System.OverflowError").unwrap(),
            fields: vec![]
        }
    );
    let old = assemble(&source.replace(
        "System.Result<Int32,System.OverflowError>",
        "System.Result<Int32,Error>",
    ))
    .unwrap();
    assert!(LoadedProgram::new(&old).unwrap().verify().is_err());
    for name in ["System.Math.Abs", "System.Int32.Divide"] {
        let function = neoclr::library::system()
            .unwrap()
            .functions
            .iter()
            .find(|f| f.name == name)
            .unwrap();
        use neoclr::metadata::Instruction::*;
        assert!(!function.is_internal_call());
        assert!(!function.body.iter().any(|op| matches!(
            op,
            Some | None(_) | Ok(_) | Err(_) | IsCase(_) | LoadCase(_)
        )));
    }
}
