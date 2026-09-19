use neoclr::{Limits, LoadedProgram, RuntimeService, Value, assembler::parse_function_ref};

fn program() -> LoadedProgram {
    let module = neoclr::assemble(".module App").unwrap();
    LoadedProgram::new(&neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap()).unwrap()
}

#[test]
fn char_predicates_observe_unicode_categories_and_unicode_scalars() {
    let p = program();
    p.verify().unwrap();
    for (name, yes, no) in [
        (
            "IsDigit",
            vec![0x30, 0x39, 0x667, 0xff19, 0x1d7ce],
            vec![0x2f, 0x3a, 0xb2, 0x2167, 0x10ffff],
        ),
        (
            "IsNumber",
            vec![0x30, 0xb2, 0x2167, 0xbd],
            vec![0x41, 0x301, 0xffff],
        ),
        (
            "IsLetter",
            vec![0x41, 0xe9, 0x4e2d, 0x1c5, 0x2b0, 0x10400],
            vec![0x301, 0x2167, 0x30, 0x10ffff],
        ),
        (
            "IsLetterOrDigit",
            vec![0x41, 0x667, 0x4e2d],
            vec![0x5f, 0xb2, 0x301],
        ),
        ("IsUpper", vec![0x41, 0xc9], vec![0x61, 0x1c5, 0x2160]),
        ("IsLower", vec![0x61, 0xe9], vec![0x41, 0x1c5, 0x2170]),
        (
            "IsWhiteSpace",
            vec![9, 10, 11, 12, 13, 0x85, 0xa0, 0x2028, 0x2029, 0x3000],
            vec![0, 8, 14, 0x200b, 0xfeff],
        ),
        (
            "IsSeparator",
            vec![0x20, 0xa0, 0x2028, 0x2029],
            vec![9, 0x85, 0x200b],
        ),
        (
            "IsControl",
            vec![0, 0x1f, 0x7f, 0x9f],
            vec![0x20, 0xa0, 0x200b],
        ),
        (
            "IsPunctuation",
            vec![0x5f, 0x2d, 0x28, 0x29, 0xab, 0xbb, 0x21],
            vec![0x2b, 0x24, 0x41],
        ),
        (
            "IsSymbol",
            vec![0x2b, 0x24, 0x5e, 0x2603, 0x1f600],
            vec![0x21, 0x41, 0x10ffff],
        ),
        ("IsAscii", vec![0, 0x7f], vec![0x80, 0xffff]),
        (
            "IsAsciiDigit",
            vec![0x30, 0x39],
            vec![0x2f, 0x3a, 0x667, 0xffff],
        ),
    ] {
        let f = p
            .resolve_function(
                &parse_function_ref(&format!("System.Text.UnicodeScalar::{name}(UInt32)")).unwrap(),
            )
            .unwrap();
        for (values, expected) in [(yes, true), (no, false)] {
            for value in values {
                assert_eq!(
                    f.invoke(vec![Value::UInt32(value)], Limits::default())
                        .unwrap()
                        .value,
                    Value::Boolean(expected),
                    "{name}: {value:04X}"
                );
            }
        }
    }
}

#[test]
fn neo_literals_and_library_calls_roundtrip() {
    let module = neoclr::frontend::compile(include_str!(
        "../examples/source/character-classification.neo"
    ))
    .unwrap();
    let p = LoadedProgram::new(&neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap())
        .unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    for (literal, expected) in [
        ("'é'", 0xe9),
        ("'🌍'", 0x1f30d),
        (r"'\U0001F600'", 0x1f600),
        (r"'\0'", 0),
        (r"'\''", 39),
        (r"'\\'", 92),
        (r"'\n'", 10),
        ("'\"'", 34),
    ] {
        let module =
            neoclr::frontend::compile(&format!("func Main() -> char {{ return {literal} }}"))
                .unwrap();
        assert_eq!(
            LoadedProgram::new(&module)
                .unwrap()
                .run(Limits::default())
                .unwrap()
                .value,
            Value::Char(char::from_u32(expected).unwrap().to_string())
        );
    }
}

#[test]
fn invalid_character_literals_have_source_diagnostics() {
    for literal in [
        "''",
        "'ab'",
        r"'\uD800'",
        r"'\U00110000'",
        r"'\x41'",
        r"'\uXYZW'",
        r"'\u123'",
        "'a\nb'",
        "'a",
    ] {
        let err = neoclr::frontend::compile(&format!("func Main() -> char {{ return {literal} }}"))
            .unwrap_err();
        assert!(err.message.contains("source 1:"), "{literal}: {err}");
    }
}

#[test]
fn ascii_checks_are_platform_il_and_unicode_checks_declare_service() {
    let p = program();
    for (name, unicode) in [
        ("IsDigit", true),
        ("IsWhiteSpace", true),
        ("IsAsciiDigit", false),
    ] {
        let graph = p
            .analyze_reachability(
                &[
                    parse_function_ref(&format!("System.Text.UnicodeScalar::{name}(UInt32)"))
                        .unwrap(),
                ],
                16,
            )
            .unwrap();
        assert_eq!(
            graph
                .required_services()
                .contains(&RuntimeService::CharacterClassification),
            unicode
        );
    }
}

#[test]
fn scalar_classification_rejects_invalid_values_and_char_rejects_integers() {
    let p = program();
    let classify = p
        .resolve_function(
            &parse_function_ref("System.Text.UnicodeScalar::IsSymbol(UInt32)").unwrap(),
        )
        .unwrap();
    for value in [0xd800, 0xdfff, 0x110000, u32::MAX] {
        assert!(
            classify
                .invoke(vec![Value::UInt32(value)], Limits::default())
                .is_err()
        );
        let module = neoclr::assemble(&format!(
            ".module Scalars\n.entry Main\n.function Main() -> Char\nldc.i4 {}\nret\n.end",
            value as i32
        ))
        .unwrap();
        assert!(
            LoadedProgram::new(&module)
                .unwrap()
                .run(Limits::default())
                .is_err()
        );
    }
}
