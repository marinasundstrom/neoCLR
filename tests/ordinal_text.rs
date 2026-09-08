use neoclr::{Limits, LoadedProgram, RuntimeService, Value, assembler::parse_function_ref};

fn program() -> LoadedProgram {
    let mut source = String::from(".module Text\n");
    for name in ["ContainsOrdinal", "StartsWithOrdinal", "EndsWithOrdinal"] {
        source.push_str(&format!(".function {name}(String text, String pattern) -> Boolean\nldarga text\nldarg pattern\ncall instance System.String::{name}(String)\nret\n.end\n"));
    }
    let module = neoclr::assemble(&source).unwrap();
    let module = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    program
}

#[test]
fn compare_preserves_dotnet_utf16_order_including_supplementary_text() {
    let program = program();
    let compare = program
        .resolve_function(
            &parse_function_ref("System.String::CompareOrdinal(String,String)").unwrap(),
        )
        .unwrap();
    for (left, right, expected) in [
        ("", "", 0),
        ("", "x", -1),
        ("x", "", 1),
        ("A", "a", -1),
        ("é", "e\u{301}", 1),
        ("\u{10000}", "\u{e000}", -1),
        ("🌍", "🌎", -1),
        ("a\0", "a", 1),
        ("abc", "abcd", -1),
    ] {
        for (left, right, expected) in [(left, right, expected), (right, left, -expected)] {
            let value = compare
                .invoke(
                    vec![Value::String(left.into()), Value::String(right.into())],
                    Limits::default(),
                )
                .unwrap()
                .value;
            let Value::Int32(actual) = value else {
                panic!("expected Int32")
            };
            assert_eq!(actual.signum(), expected, "{left:?} vs {right:?}");
        }
    }
}

#[test]
fn predicates_match_exact_text_with_empty_nul_and_combining_marks() {
    let program = program();
    for (text, pattern, contains, starts, ends) in [
        ("", "", true, true, true),
        ("abc", "", true, true, true),
        ("", "x", false, false, false),
        ("café🌍", "é", true, false, false),
        ("café🌍", "café", true, true, false),
        ("café🌍", "🌍", true, false, true),
        ("a\0b", "\0", true, false, false),
        ("é", "e\u{301}", false, false, false),
        ("e\u{301}", "\u{301}", true, false, true),
        ("Neo", "neo", false, false, false),
        ("x", "xx", false, false, false),
    ] {
        for (name, expected) in [
            ("ContainsOrdinal", contains),
            ("StartsWithOrdinal", starts),
            ("EndsWithOrdinal", ends),
        ] {
            let function = program
                .resolve_function(&parse_function_ref(&format!("{name}(String,String)")).unwrap())
                .unwrap();
            assert_eq!(
                function
                    .invoke(
                        vec![Value::String(text.into()), Value::String(pattern.into())],
                        Limits::default()
                    )
                    .unwrap()
                    .value,
                Value::Boolean(expected),
                "{name}: {text:?}, {pattern:?}"
            );
        }
    }
}

#[test]
fn neo_sample_combines_eager_search_and_text_through_artifact() {
    let module =
        neoclr::frontend::compile(include_str!("../examples/source/ordinal-text.neo")).unwrap();
    let module = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    let result = program.run(Limits::default()).unwrap();
    assert_eq!(result.output, ["café.neo"]);
    assert_eq!(result.value, Value::Int32(42));
}

#[test]
fn service_planning_covers_all_ordinal_helpers() {
    let program = program();
    for name in ["ContainsOrdinal", "StartsWithOrdinal", "EndsWithOrdinal"] {
        let graph = program
            .analyze_reachability(
                &[parse_function_ref(&format!("instance System.String::{name}(String)")).unwrap()],
                8,
            )
            .unwrap();
        assert!(
            graph
                .required_services()
                .contains(&RuntimeService::StringOperations)
        );
        assert!(
            graph
                .required_services()
                .contains(&RuntimeService::SlotReferences)
        );
    }
    let graph = program
        .analyze_reachability(
            &[parse_function_ref("System.String::CompareOrdinal(String,String)").unwrap()],
            8,
        )
        .unwrap();
    assert_eq!(
        graph.required_services(),
        [RuntimeService::StringOperations]
    );
}

#[test]
fn neo_borrows_temporary_and_readonly_reference_receivers() {
    let module = neoclr::frontend::compile(
        r#"
func Check(text: readonly string&) -> bool {
    return text.StartsWithOrdinal("café") && text.EndsWithOrdinal("🌍")
}
func Main() -> int {
    let text = "café🌍"
    if !Check(&text) { return -1 }
    if !System.String.Concat("ca", "fé").ContainsOrdinal("é") { return -2 }
    if !"".EndsWithOrdinal("") { return -3 }
    return 42
}
"#,
    )
    .unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}
