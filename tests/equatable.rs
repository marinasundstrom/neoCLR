use neoclr::{Limits, LoadedProgram, Value, assemble, load};

fn program(extra: &str, body: &str) -> LoadedProgram {
    let module = assemble(&format!(
        ".module Equality\n.entry Main\n{extra}\n.function Main() -> Boolean\n{body}\nret\n.end"
    ))
    .unwrap();
    let program =
        LoadedProgram::new(&load(&serde_json::to_string(&module).unwrap()).unwrap()).unwrap();
    program.verify().unwrap();
    program
}

#[test]
fn sample_compares_records_and_primitives_through_managed_views() {
    let module = assemble(include_str!("../examples/equatable.neoil")).unwrap();
    let p = LoadedProgram::new(&load(&serde_json::to_string(&module).unwrap()).unwrap()).unwrap();
    p.verify().unwrap();
    let execution = p.run(Limits::default()).unwrap();
    assert_eq!(execution.output, ["true", "false", "true", "true"]);
    assert_eq!(execution.memory.live_allocations(), 0);
}

#[test]
fn library_equals_agrees_between_direct_and_interface_calls() {
    for (ty, left, right, expected) in [
        ("Int32", "ldc.i4 -42", "ldc.i4 -42", true),
        ("Int32", "ldc.i4 0", "ldc.i4 1", false),
        ("String", "ldstr \"héllo\"", "ldstr \"héllo\"", true),
        ("String", "ldstr \"Hello\"", "ldstr \"hello\"", false),
        ("String", "ldstr \"é\"", "ldstr \"é\"", false),
        (
            "System.Type",
            "ldtoken Int32\ncall System.Type::GetTypeFromHandle(System.RuntimeTypeHandle)",
            "ldtoken Int32\ncall System.Type::GetTypeFromHandle(System.RuntimeTypeHandle)",
            true,
        ),
        (
            "System.Type",
            "ldtoken Int32\ncall System.Type::GetTypeFromHandle(System.RuntimeTypeHandle)",
            "ldtoken String\ncall System.Type::GetTypeFromHandle(System.RuntimeTypeHandle)",
            false,
        ),
    ] {
        for interface in [false, true] {
            let (receiver, call) = if interface {
                (
                    format!("ldloca value\ninterface.borrow System.Equatable<{ty}>"),
                    format!("callvirt instance System.Equatable<{ty}>::Equals({ty})"),
                )
            } else {
                (
                    "ldloc value".into(),
                    format!("call instance {ty}::Equals({ty})"),
                )
            };
            let body =
                format!(".local {ty} value\n{left}\nstloc value\n{receiver}\n{right}\n{call}");
            assert_eq!(
                program("", &body).run(Limits::default()).unwrap().value,
                Value::Boolean(expected)
            );
        }
    }
}

#[test]
fn generic_implementations_can_compare_a_different_type() {
    let extra = r#"
.type Tagged<T>
    .implements System.Equatable<Int32>
    .field Tag Int32
    .field Payload T
    .method instance Equals(Int32 other) -> Boolean
        ldarg this
        ldfld Tagged<T>::Tag
        ldarg other
        ceq
        ret
    .end
.end
"#;
    for (other, expected) in [(42, true), (7, false)] {
        let body = format!(
            ".local Tagged<String> value\nldc.i4 42\nldstr \"payload\"\nnewobj Tagged<String>\nstloc value\nldloca value\ninterface.borrow System.Equatable<Int32>\nldc.i4 {other}\ncallvirt instance System.Equatable<Int32>::Equals(Int32)"
        );
        assert_eq!(
            program(extra, &body).run(Limits::default()).unwrap().value,
            Value::Boolean(expected)
        );
    }
}
