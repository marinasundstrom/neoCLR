use neoclr::{Limits, Value, assemble, run, verify};

fn module(functions: &str) -> neoclr::Module {
    assemble(&format!(".module NoResult\n.entry Main\n{functions}")).unwrap()
}

#[test]
fn nested_no_result_calls_preserve_caller_stack() {
    let m = module(
        ".function Main() -> Int32\nldc.i4 42\ncall Outer()\nret\n.end\n.function Outer() -> noresult\ncall Inner()\nret\n.end\n.function Inner() -> noresult\nret\n.end",
    );
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(42));
}

#[test]
fn no_result_entry_can_call_the_existing_system_library() {
    let m = module(
        ".function Main() -> noresult\nldstr \"hello\"\ncall System.Console::WriteLine(String)\npop\nret\n.end",
    );
    verify(&m).unwrap();
    let execution = run(&m, Limits::default()).unwrap();
    assert_eq!(execution.output, ["hello"]);
    assert_eq!(execution.value, Value::Void); // host envelope, not a guest stack result
}

#[test]
fn result_is_required_only_by_value_returning_methods() {
    for (body, message) in [
        (
            ".function Main() -> noresult\nldvoid\nret\n.end",
            "empty stack",
        ),
        (
            ".function Main() -> noresult\ncall Empty()\npop\nret\n.end\n.function Empty() -> noresult\nret\n.end",
            "underflow",
        ),
        (".function Main() -> Void\nret\n.end", "underflow"),
    ] {
        let m = module(body);
        assert!(verify(&m).unwrap_err().message.contains(message));
        assert!(run(&m, Limits::default()).is_err());
    }
}

#[test]
fn serde_preserves_return_mode_and_rejects_invalid_mode_metadata() {
    let m = module(".function Main() -> noresult\nret\n.end");
    let json = serde_json::to_string(&m).unwrap();
    let mut decoded: neoclr::Module = serde_json::from_str(&json).unwrap();
    assert!(decoded.functions[0].no_result);
    verify(&decoded).unwrap();
    decoded.functions[0].returns = neoclr::metadata::Type::Int32;
    assert!(verify(&decoded).unwrap_err().message.contains("no-result"));
    assert!(
        run(&decoded, Limits::default())
            .unwrap_err()
            .message
            .contains("no-result")
    );
}

#[test]
fn void_remains_an_inhabited_generic_argument() {
    let m = module(
        ".function Main() -> Void\nldvoid\ncall Identity<Void>(Void)\nret\n.end\n.function Identity<T>(T) -> T\nldarg 0\nret\n.end",
    );
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Void);
    assert!(!m.functions[1].no_result);
}

#[test]
fn generic_storage_can_contain_void_inside_a_no_result_method() {
    let m = module(
        ".type Box<T>\n.field Value T\n.end\n.function Main() -> noresult\n.local Box<Void> box\nldvoid\nnewobj Box<Void>\nstloc box\nret\n.end",
    );
    verify(&m).unwrap();
    run(&m, Limits::default()).unwrap();
}

#[test]
fn inhabited_void_delegate_cannot_bind_a_no_result_target() {
    let mut m = module(
        ".delegate Action\n.method instance Invoke() -> Void\n.end\n.end\n.function Main() -> Void\ndelegate.bind Action = Empty()\npop\nldvoid\nret\n.end\n.function Empty() -> Void\nldvoid\nret\n.end",
    );
    let empty = m.functions.iter_mut().find(|f| f.name == "Empty").unwrap();
    empty.no_result = true;
    empty.body = vec![neoclr::metadata::Instruction::Return];
    assert!(verify(&m).is_err());
    assert!(run(&m, Limits::default()).is_err());
}
