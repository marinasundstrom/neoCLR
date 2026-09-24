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

#[test]
fn generic_no_result_calls_preserve_caller_values_and_void_payloads() {
    let m = module(
        ".function Main() -> Int32\nldc.i4 42\nldvoid\ncall Ignore<Void>(Void)\nldc.i4 7\ncall Ignore<Int32>(Int32)\nret\n.end\n.function Ignore<T>(T value) -> noresult\nldarg value\npop\nret\n.end",
    );
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(42));
}

#[test]
fn generic_no_result_body_rejects_a_return_value() {
    let m = module(
        ".function Main() -> noresult\nldc.i4 7\ncall Bad<Int32>(Int32)\nret\n.end\n.function Bad<T>(T value) -> noresult\nldarg value\nret\n.end",
    );
    assert!(verify(&m).unwrap_err().message.contains("empty stack"));
    assert!(run(&m, Limits::default()).is_err());
}

#[test]
fn cli_void_return_spelling_preserves_stack_and_named_unit_returns() {
    let m = module(
        ".function Main() -> Int32\nldc.i4 42\ncall Empty()\ncall Unit()\npop\nret\n.end\n.function Empty() -> void\nret\n.end\n.function Unit() -> System.Void\nldvoid\nret\n.end",
    );
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(42));
    assert!(
        m.functions
            .iter()
            .find(|f| f.name == "Empty")
            .unwrap()
            .no_result
    );
    assert!(
        !m.functions
            .iter()
            .find(|f| f.name == "Unit")
            .unwrap()
            .no_result
    );
}

#[test]
fn cli_void_return_rejects_unit_on_stack_but_accepts_unit_generic_storage() {
    let bad = module(".function Main() -> void\nldvoid\nret\n.end");
    assert!(verify(&bad).is_err());
    assert!(run(&bad, Limits::default()).is_err());
    let good = module(
        ".type Box<T>\n.field Value T\n.end\n.function Main() -> void\n.local Box<System.Void> box\nldvoid\nnewobj Box<System.Void>\nstloc box\nret\n.end",
    );
    verify(&good).unwrap();
    run(&good, Limits::default()).unwrap();
}

#[test]
fn value_no_result_method_mutates_local_and_box_without_stack_payload() {
    let m = module(r#"
.type class System.Object
.end
.interface Stepper
.method instance Step() -> noresult
.end
.end
.type State
.implements Stepper
.field Number Int32
.method instance byref Step() -> noresult
ldarg this
ldflda 0
ldarg this
ldfld 0
ldc.i4 1
add
stobj Int32
ret
.end
.end
.function Main() -> Int32
.local State state
.local Stepper boxed
ldc.i4 40
newobj State
stloc state
ldloca state
call instance State::Step()
ldloc state
box State
castclass Stepper
stloc boxed
ldc.i4 42
ldloc boxed
callvirt instance Stepper::Step()
pop
ldloc boxed
unbox.any State
ldfld State::Number
ret
.end
"#);
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(42));
}
