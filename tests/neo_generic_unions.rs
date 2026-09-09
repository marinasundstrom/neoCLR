use neoclr::{Limits, LoadedProgram, Value, frontend};

fn run(source: &str) -> neoclr::Execution {
    let module = frontend::compile(source).unwrap();
    let loaded = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    let program = LoadedProgram::new(&loaded).unwrap();
    program.verify().unwrap();
    program.run(Limits::default()).unwrap()
}

const DECLARATIONS: &str = r#"
record Success<T>(Value: T)
record Failure<E>(Error: E)
union Outcome<T,E>(Success<T> | Failure<E>)
func Wrap<T>(value: T) -> Outcome<T,string> { return Success<T>(value) }
func Read(result: Outcome<int,string>) -> int {
    return result match {
        Success(let ok) => ok.Value,
        Failure(_) => -1
    }
}
"#;

#[test]
fn generic_carriers_construct_convert_and_match_after_artifact_loading() {
    let source = format!(
        "{DECLARATIONS}{}",
        r#"
func Main() -> int {
    let success = Success<int>(20)
    let result: Outcome<int,string> = success
    let explicit = Outcome<int,string>(Success<int>(22))
    let failed: Outcome<int,string> = Failure<string>("failed")
    if let Success(unexpected) = failed { return -2 }
    let Failure(error) = failed else { return -3 }
    if !error.Error.Equals("failed") { return -4 }
    let forwarded = Wrap(42)
    if Read(forwarded) != 42 { return -5 }
    return Read(result) + Read(explicit)
}
"#
    );
    assert_eq!(run(&source).value, Value::Int32(42));
    let module = frontend::compile(&source).unwrap();
    let carrier = module.types.iter().find(|t| t.name == "Outcome").unwrap();
    assert_eq!(carrier.generic_parameters.len(), 2);
    assert_eq!(
        module.types.iter().filter(|t| t.name == "Outcome").count(),
        1
    );
}

#[test]
fn cases_keep_independent_parameters_and_reference_identity() {
    let source = format!(
        "{DECLARATIONS}{}",
        r#"
record Cell(Value: int)
func Main() -> int {
    let cell = new Cell(40)
    let result: Outcome<Cell&,string> = Success<Cell&>(cell)
    let copy = result
    let Success(ok) = copy else { return -1 }
    if !ReferenceEquals(ok.Value, cell) { return -2 }
    ok.Value.Value = 42
    let readonlyCell: readonly Cell& = cell
    let readonlyResult: Outcome<readonly Cell&,int> = Success<readonly Cell&>(readonlyCell)
    let Success(view) = readonlyResult else { return -3 }
    return view.Value.Value
}
"#
    );
    assert_eq!(run(&source).value, Value::Int32(42));
}

#[test]
fn nested_carriers_and_empty_cases_work() {
    assert_eq!(
        run(r#"
record Some<T>(Value: T)
record None()
union Maybe<T>(Some<T> | None)
record Failure<E>(Error: E)
union Nested<T>(Maybe<T> | Failure<string>)
func Main() -> int {
    let inner: Maybe<int> = Some<int>(42)
    let outer: Nested<int> = inner
    let Maybe(value) = outer else { return -1 }
    let Some(answer) = value else { return -2 }
    let empty: Maybe<int> = None()
    if let Some(unexpected) = empty { return -3 }
    return answer.Value
}
"#)
        .value,
        Value::Int32(42)
    );
}

#[test]
fn invalid_generic_union_contracts_are_rejected() {
    for source in [
        "record Case<T>(Value: T)\nunion Bad<T>(Case)\nfunc Main() -> int { return 0 }",
        "record Case<T>(Value: T)\nunion Bad<T>(Case<T> | Case<int>)\nfunc Main() -> int { return 0 }",
        "union Bad<T>(Bad<T>)\nfunc Main() -> int { return 0 }",
        "union Bad<T> { case Case(Value: T) }\nfunc Main() -> int { return 0 }",
        "record Case<T>(Value: T)\nunion Bad<T>(Case<Unknown>)\nfunc Main() -> int { return 0 }",
    ] {
        assert!(frontend::compile(source).is_err(), "accepted {source}");
    }
    for body in [
        "return Read(Success<string>(\"wrong\"))",
        "let result = Outcome(Success<int>(42))\nreturn 0",
        "let result = Outcome<int>(Success<int>(42))\nreturn 0",
        "let result = default(Outcome<int,string>)\nreturn 0",
        "let result: Outcome<int,string> = Success<int>(42)\nreturn result match { Success(let ok) => ok.Value }",
    ] {
        let source = format!("{DECLARATIONS}\nfunc Main() -> int {{ {body} }}");
        assert!(frontend::compile(&source).is_err(), "accepted {body}");
    }
}

#[test]
fn example_runs() {
    assert_eq!(
        run(include_str!("../examples/source/generic-unions.neo")).value,
        Value::Int32(42)
    );
}

#[test]
fn generic_carrier_cannot_hide_frame_references() {
    let source = r#"
record Cell(Value: int)
record Case<T>(Value: T)
union Carrier<T>(Case<T>)
func Main() -> int {
    var cell = Cell(42)
    let carrier: Carrier<Cell&> = Case<Cell&>(&cell)
    return 0
}
"#;
    let module = frontend::compile(source).unwrap();
    assert!(
        LoadedProgram::new(&module)
            .unwrap()
            .run(Limits::default())
            .is_err()
    );
}

#[test]
fn case_constraints_and_parameter_shadowing_remain_effective() {
    assert_eq!(
        run(r#"
import System.Option.*
record Case<T>(Value: T) where T: notvoid
union Carrier<Some>(Case<Some>)
func Main() -> int {
    let carrier: Carrier<int> = Case<int>(42)
    let Case(value) = carrier else { return -1 }
    return value.Value
}
"#)
        .value,
        Value::Int32(42)
    );
    assert!(
        frontend::compile(
            r#"
record Case<T>(Value: T) where T: notvoid
union Carrier<T>(Case<T>)
func Main() -> int { let bad: Carrier<void> = Case<void>(default(void))
return 0 }
"#
        )
        .is_err()
    );
}
