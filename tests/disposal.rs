use neoclr::{Limits, LoadedProgram, Value, assemble, load};

const SAMPLE: &str = include_str!("../examples/disposal.neoil");

fn run(body: &str) -> neoclr::Execution {
    let (definitions, _) = SAMPLE.split_once(".function Main() -> Void").unwrap();
    let source = format!("{definitions}.function Main() -> Draft\n{body}\nret\n.end");
    let module = assemble(&source).unwrap();
    let artifact = serde_json::to_string(&module).unwrap();
    let program = LoadedProgram::new(&load(&artifact).unwrap()).unwrap();
    program.verify().unwrap();
    program.run(Limits::default()).unwrap()
}

fn fields(execution: neoclr::Execution) -> Vec<Value> {
    let Value::Object { fields, .. } = execution.value else {
        panic!("expected draft");
    };
    fields
}

#[test]
fn sample_closes_and_handles_failure_before_explicit_disposal() {
    let module = assemble(SAMPLE).unwrap();
    let artifact = serde_json::to_string(&module).unwrap();
    let program = LoadedProgram::new(&load(&artifact).unwrap()).unwrap();
    program.verify().unwrap();
    let execution = program.run(Limits::default()).unwrap();
    assert_eq!(
        execution.output,
        ["Draft closed", "1", "Draft is empty", "1"]
    );
    assert_eq!(execution.value, Value::Void);
}

#[test]
fn repeated_disposal_updates_only_the_explicitly_borrowed_value() {
    let result = fields(run(r#"
        .local Draft original
        .local Draft copied
        ldstr "kept"
        call NewDraft(String)
        stloc original
        ldloc original
        stloc copied
        ldloca copied
        interface.borrow System.Disposable
        call Release(System.Disposable&)
        pop
        ldloca copied
        interface.borrow System.Disposable
        call Release(System.Disposable&)
        pop
        ldloc copied
        ldfld Draft::ReleaseCount
        ldc.i4 1
        bne.un Invalid
        ldloc copied
        ldfld Draft::Text
        call instance System.String::IsEmpty()
        brfalse Invalid
        ldloc original
        ret
    Invalid:
        fault "Dispose must release once and clear the copied draft"
        "#));
    assert_eq!(
        result,
        [
            Value::String("kept".into()),
            Value::Boolean(false),
            Value::Boolean(false),
            Value::Int32(0)
        ]
    );
}

#[test]
fn closing_is_repeatable_and_does_not_implicitly_dispose() {
    let result = fields(run(r#"
        .local Draft value
        ldstr "ready"
        call NewDraft(String)
        stloc value
        ldloca value
        interface.borrow System.Closable<DraftCloseError>
        call Finish(System.Closable<DraftCloseError>&)
        call instance System.Result<Void,DraftCloseError>::GetOkCase()
        pop
        ldloca value
        interface.borrow System.Closable<DraftCloseError>
        call Finish(System.Closable<DraftCloseError>&)
        call instance System.Result<Void,DraftCloseError>::GetOkCase()
        pop
        ldloc value
        "#));
    assert_eq!(
        result,
        [
            Value::String("ready".into()),
            Value::Boolean(true),
            Value::Boolean(false),
            Value::Int32(0)
        ]
    );
}

#[test]
fn failed_close_can_be_followed_by_disposal_and_typed_closed_state_error() {
    let result = fields(run(r#"
        .local Draft value
        ldstr ""
        call NewDraft(String)
        stloc value
        ldloca value
        interface.borrow System.Closable<DraftCloseError>
        call Finish(System.Closable<DraftCloseError>&)
        call instance System.Result<Void,DraftCloseError>::GetErrorCase()
        pop
        ldloca value
        interface.borrow System.Disposable
        call Release(System.Disposable&)
        pop
        ldloca value
        interface.borrow System.Closable<DraftCloseError>
        call Finish(System.Closable<DraftCloseError>&)
        call instance System.Result<Void,DraftCloseError>::GetErrorCase()
        call instance System.Result.Error<DraftCloseError>::get_Value()
        ldfld DraftCloseError::Message
        ldstr "Draft is disposed"
        ceq
        brfalse Invalid
        ldloc value
        ret
    Invalid:
        fault "expected disposed error"
        "#));
    assert_eq!(
        result,
        [
            Value::String("".into()),
            Value::Boolean(false),
            Value::Boolean(true),
            Value::Int32(1)
        ]
    );
}

#[test]
fn cleanup_interfaces_require_exact_byref_and_result_contracts() {
    for source in [
        SAMPLE.replace("instance byref Dispose", "instance Dispose"),
        SAMPLE.replace("instance byref Close", "instance Close"),
        SAMPLE.replace(
            "Close() -> System.Result<Void,DraftCloseError>",
            "Close() -> Void",
        ),
    ] {
        assert!(assemble(&source).is_err());
    }
}
