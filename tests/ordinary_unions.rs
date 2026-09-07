use neoclr::{Limits, LoadedProgram, Value, assemble, metadata::Type};

const SAMPLE: &str = include_str!("../examples/ordinary_unions.neoil");

fn evaluate(body: &str, returns: &str) -> Result<Value, neoclr::Fault> {
    let module = assemble(&format!(
        ".module App\n.entry Main\n.function Main() -> {returns}\n{body}\nret\n.end"
    ))?;
    let loaded = LoadedProgram::new(&module)?;
    loaded.verify()?;
    Ok(loaded.run(Limits::default())?.value)
}

#[test]
fn ordinary_system_carriers_round_trip_and_distinguish_equal_payload_types() {
    let module = assemble(SAMPLE).unwrap();
    let module = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    let loaded = LoadedProgram::new(&module).unwrap();
    loaded.verify().unwrap();
    assert_eq!(
        loaded.run(Limits::default()).unwrap().output,
        ["success", "7", "failure", "7", "Some<Void> is present"]
    );
}

#[test]
fn none_and_some_void_are_distinct_and_queries_agree() {
    for (creation, some) in [
        (
            "newobj instance System.Option.None::.ctor()\nnewobj instance System.Option<Void>::.ctor(System.Option.None)",
            false,
        ),
        (
            "ldvoid\nnewobj instance System.Option.Some<Void>::.ctor(Void)\nnewobj instance System.Option<Void>::.ctor(System.Option.Some<Void>)",
            true,
        ),
    ] {
        for (query, expected) in [("IsSome", some), ("IsNone", !some)] {
            assert_eq!(
                evaluate(
                    &format!("{creation}\ncall instance System.Option<Void>::get_{query}()"),
                    "Boolean"
                )
                .unwrap(),
                Value::Boolean(expected)
            );
        }
    }
    assert_eq!(evaluate("ldvoid\nnewobj instance System.Option.Some<Void>::.ctor(Void)\nnewobj instance System.Option<Void>::.ctor(System.Option.Some<Void>)\ncall instance System.Option<Void>::GetSomeCase()\ncall instance System.Option.Some<Void>::get_Value()", "Void").unwrap(), Value::Void);
}

#[test]
fn void_success_and_arbitrary_error_payloads_use_ordinary_wrappers() {
    assert_eq!(evaluate("ldvoid\nnewobj instance System.Result.Ok<Void>::.ctor(Void)\nnewobj instance System.Result<Void,String>::.ctor(System.Result.Ok<Void>)\ncall instance System.Result<Void,String>::GetOkCase()\ncall instance System.Result.Ok<Void>::get_Value()", "Void").unwrap(), Value::Void);
    assert_eq!(evaluate("ldstr \"failure text\"\nnewobj instance System.Result.Error<String>::.ctor(String)\nnewobj instance System.Result<Void,String>::.ctor(System.Result.Error<String>)\ncall instance System.Result<Void,String>::GetErrorCase()\ncall instance System.Result.Error<String>::get_Value()", "String").unwrap(), Value::String("failure text".into()));
}

#[test]
fn nested_option_result_preserves_the_complete_closed_payload() {
    let body = "ldc.i4 257\nnewobj instance System.Option.Some<Byte>::.ctor(Byte)\nnewobj instance System.Option<Byte>::.ctor(System.Option.Some<Byte>)\nnewobj instance System.Result.Ok<System.Option<Byte>>::.ctor(System.Option<Byte>)\nnewobj instance System.Result<System.Option<Byte>,Error>::.ctor(System.Result.Ok<System.Option<Byte>>)\ncall instance System.Result<System.Option<Byte>,Error>::GetOkCase()\ncall instance System.Result.Ok<System.Option<Byte>>::get_Value()\ncall instance System.Option<Byte>::GetSomeCase()\ncall instance System.Option.Some<Byte>::get_Value()";
    assert_eq!(evaluate(body, "Int32").unwrap(), Value::Int32(1));
}

#[test]
fn wrong_accessor_faults_without_fabricating_a_payload() {
    let fault = evaluate("newobj instance System.Option.None::.ctor()\nnewobj instance System.Option<Int32>::.ctor(System.Option.None)\ncall instance System.Option<Int32>::GetSomeCase()", "System.Option.Some<Int32>").unwrap_err();
    assert!(fault.message.contains("erased value contains"));
    let trace = fault.stack_trace.unwrap();
    assert_eq!(trace.frames[0].function.name, "System.Option.GetSomeCase");
    assert_eq!(trace.frames[1].function.name, "Main");
}

#[test]
fn representation_is_private_and_extracted_records_are_independent_copies() {
    let bad = ".module App\n.function Main() -> System.Option<Int32>\nldc.i4 42\nvalue.pack Int32\nnewobj System.Option<Int32>\nret\n.end";
    assert!(
        assemble(bad)
            .unwrap_err()
            .message
            .contains("field access denied")
    );
    let src = ".module App\n.entry Main\n.type Cell\n.field Number Int32\n.end\n.function Main() -> Int32\n.local System.Option<Cell> saved\nldc.i4 7\nnewobj Cell\nnewobj instance System.Option.Some<Cell>::.ctor(Cell)\nnewobj instance System.Option<Cell>::.ctor(System.Option.Some<Cell>)\nstloc saved\nldloc saved\ncall instance System.Option<Cell>::GetSomeCase()\ncall instance System.Option.Some<Cell>::get_Value()\nldc.i4 42\nstfld 0\npop\nldloc saved\ncall instance System.Option<Cell>::GetSomeCase()\ncall instance System.Option.Some<Cell>::get_Value()\nldfld 0\nret\n.end";
    let module = assemble(src).unwrap();
    let loaded = LoadedProgram::new(&module).unwrap();
    loaded.verify().unwrap();
    assert_eq!(
        loaded.run(Limits::default()).unwrap().value,
        Value::Int32(7)
    );
}

#[test]
fn carrier_execution_does_not_depend_on_markers_or_bootstrap_union_opcodes() {
    let mut library = neoclr::library::system().unwrap().clone();
    for ty in &mut library.types {
        ty.custom_attributes.clear();
    }
    let new_methods = library.functions.iter().filter(|f| {
        f.name.starts_with("System.Option.")
            || f.name.starts_with("System.Result.")
            || f.name.starts_with("System.Option.Some.")
            || f.name.starts_with("System.Option.None.")
            || f.name.starts_with("System.Result.Ok.")
            || f.name.starts_with("System.Result.Error.")
    });
    let mut count = 0;
    for method in new_methods {
        count += 1;
        assert!(!method.is_internal_call());
    }
    assert!(count > 0, "carrier method checks must not be vacuous");
    let module = assemble(SAMPLE).unwrap();
    let loaded = LoadedProgram::with_library(&module, &library).unwrap();
    loaded.verify().unwrap();
    assert_eq!(loaded.run(Limits::default()).unwrap().output.len(), 5);
}

#[test]
fn unqualified_carrier_names_have_no_special_runtime_meaning() {
    assert!(
        matches!(neoclr::assembler::parse_type("Option<Int32>").unwrap(), Type::Constructed { definition, .. } if definition == "Option")
    );
    assert!(
        matches!(neoclr::assembler::parse_type("Result<Int32,Error>").unwrap(), Type::Constructed { definition, .. } if definition == "Result")
    );
}

#[test]
fn bundled_carriers_only_accept_the_nested_case_family() {
    let library = neoclr::library::system().unwrap();
    for name in ["System.None", "System.Some", "System.Ok", "System.Err"] {
        assert!(!library.types.iter().any(|ty| ty.name == name));
    }
    for name in [
        "System.Option.GetNone",
        "System.Option.GetSome",
        "System.Result.GetOk",
        "System.Result.GetErr",
    ] {
        assert!(
            !library
                .functions
                .iter()
                .any(|function| function.name == name)
        );
    }
    assert!(assemble(".module App\n.function F() -> Void\nnewobj instance System.None::.ctor()\npop\nldvoid\nret\n.end").is_err());
    assert!(assemble(".module App\n.function F(System.Result<Int32,Error> value) -> Void\nldarg value\ncall instance System.Result<Int32,Error>::GetOk()\npop\nldvoid\nret\n.end").is_err());
}
