use neoclr::{Limits, LoadedProgram, RuntimeService, Value, assembler::parse_function_ref};

#[test]
fn lexical_path_contract_roundtrips_through_library_and_artifact() {
    let module = neoclr::frontend::compile(
        r#"
func Main() -> int {
    let path = System.IO.Path.Combine("reports", "result.txt")
    let name = System.IO.Path.GetFileName(path)
    if !name.Equals("result.txt") { return 1 }
    return 0
}
"#,
    )
    .unwrap();
    let p = LoadedProgram::new(&neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap())
        .unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(0));
    let combine = p
        .resolve_function(&parse_function_ref("System.IO.Path::Combine(String,String)").unwrap())
        .unwrap();
    for (left, right, expected) in [
        ("", "a", "a".to_owned()),
        ("a", "", "a".to_owned()),
        ("a/", "b", "a/b".to_owned()),
        ("a", "/b", "/b".to_owned()),
        (
            "a",
            "../é.txt",
            format!("a{}../é.txt", std::path::MAIN_SEPARATOR),
        ),
        ("a", "x\0y", format!("a{}x\0y", std::path::MAIN_SEPARATOR)),
    ] {
        assert_eq!(
            combine
                .invoke(
                    vec![Value::String(left.into()), Value::String(right.into())],
                    Limits::default()
                )
                .unwrap()
                .value,
            Value::String(expected)
        );
    }
    let filename = p
        .resolve_function(&parse_function_ref("System.IO.Path::GetFileName(String)").unwrap())
        .unwrap();
    for (path, expected) in [
        ("", ""),
        ("/", ""),
        ("a/", ""),
        ("a/..", ".."),
        ("a/é.txt", "é.txt"),
        ("a//b", "b"),
    ] {
        assert_eq!(
            filename
                .invoke(vec![Value::String(path.into())], Limits::default())
                .unwrap()
                .value,
            Value::String(expected.into())
        );
    }
    #[cfg(unix)]
    {
        assert_eq!(
            filename
                .invoke(vec![Value::String("a\\b".into())], Limits::default())
                .unwrap()
                .value,
            Value::String("a\\b".into())
        );
    }
    #[cfg(windows)]
    {
        for (path, expected) in [
            ("C:foo", "foo"),
            ("C:\\foo", "foo"),
            ("\\\\server\\share", ""),
            ("C:\\", ""),
        ] {
            assert_eq!(
                filename
                    .invoke(vec![Value::String(path.into())], Limits::default())
                    .unwrap()
                    .value,
                Value::String(expected.into())
            );
        }
        assert_eq!(
            combine
                .invoke(
                    vec![Value::String("a".into()), Value::String("C:foo".into())],
                    Limits::default()
                )
                .unwrap()
                .value,
            Value::String("C:foo".into())
        );
    }
    assert_eq!(
        p.analyze_reachability(
            &[parse_function_ref("System.IO.Path::Combine(String,String)").unwrap()],
            8
        )
        .unwrap()
        .required_services(),
        vec![RuntimeService::PathOperations]
    );
}
