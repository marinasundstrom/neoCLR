use neoclr::{Limits, LoadedProgram, Value, assemble, assembler::parse_function_ref};

fn program() -> LoadedProgram {
    let mut source =
        ".module Text\n.function Echo(Char value) -> Char\nldarg value\nret\n.end\n".to_owned();
    for (name, result) in [
        ("StringGraphemeCount", "Int32"),
        ("StringGraphemes", "Char[]"),
        ("StringScalars", "UInt32[]"),
    ] {
        source.push_str(&format!(".function {name}(String text) -> {result}\nldarg text\ncall neoCLR.Runtime.{name}(String)\nret\n.end\n"));
    }
    LoadedProgram::new(&assemble(&source).unwrap()).unwrap()
}
fn invoke(signature: &str, args: Vec<Value>) -> Result<Value, neoclr::Fault> {
    program()
        .resolve_function(&parse_function_ref(signature).unwrap())
        .unwrap()
        .invoke(args, Limits::default())
        .map(|e| e.value)
}
#[test]
fn grapheme_construction_preserves_text_and_validates_host_values() {
    assert_eq!(unicode_segmentation::UNICODE_VERSION, (16, 0, 0));
    for text in ["A", "e\u{301}", "é", "👨‍👩‍👧‍👦", "🇸🇪", "👍🏽", "\r\n", "\0"]
    {
        let value = Value::Char(text.into());
        assert_eq!(
            invoke(
                "System.Char::FromString(String)",
                vec![Value::String(text.into())]
            )
            .unwrap(),
            value
        );
        assert_eq!(invoke("Echo(Char)", vec![value.clone()]).unwrap(), value);
    }
    for text in ["", "ab", "🇸🇪🇩🇰", "a\0"] {
        assert!(
            invoke(
                "System.Char::FromString(String)",
                vec![Value::String(text.into())]
            )
            .is_err()
        );
        assert!(invoke("Echo(Char)", vec![Value::Char(text.into())]).is_err());
    }
}
#[test]
fn grapheme_and_scalar_snapshots_have_distinct_units() {
    let text = Value::String("Aé👨‍👩‍👧‍👦🇸🇪".into());
    assert_eq!(
        invoke("StringGraphemeCount(String)", vec![text.clone()]).unwrap(),
        Value::Int32(4)
    );
    let Value::Array { elements, .. } =
        invoke("StringGraphemes(String)", vec![text.clone()]).unwrap()
    else {
        panic!()
    };
    assert_eq!(
        elements,
        ["A", "é", "👨‍👩‍👧‍👦", "🇸🇪"].map(|s| Value::Char(s.into()))
    );
    let Value::Array { elements, .. } = invoke("StringScalars(String)", vec![text]).unwrap() else {
        panic!()
    };
    assert_eq!(elements.len(), 12);
    assert_eq!(
        &elements[..3],
        &[Value::UInt32(65), Value::UInt32(101), Value::UInt32(769)]
    );
    assert_eq!(
        invoke(
            "StringGraphemeCount(String)",
            vec![Value::String("".into())]
        )
        .unwrap(),
        Value::Int32(0)
    );
}
#[test]
fn character_has_no_integer_or_native_inline_representation() {
    for body in ["ldc.i4 65", "ldc.i4 1\nheap.alloc Char\nldobj Char"] {
        let result = assemble(&format!(
            ".module Invalid\n.entry Main\n.function Main() -> Char\n{body}\nret\n.end"
        ))
        .and_then(|module| LoadedProgram::new(&module))
        .and_then(|program| program.run(Limits::default()));
        assert!(result.is_err());
    }
}
#[test]
fn segmentation_observes_concatenation_context_without_normalization() {
    assert_eq!(
        invoke(
            "StringGraphemeCount(String)",
            vec![Value::String("e\u{301}".into())]
        )
        .unwrap(),
        Value::Int32(1)
    );
    assert_ne!(Value::Char("é".into()), Value::Char("e\u{301}".into()));
}

#[test]
fn character_snapshots_enforce_element_and_payload_limits() {
    let p = program();
    let snapshot = p
        .resolve_function(&parse_function_ref("StringGraphemes(String)").unwrap())
        .unwrap();
    let limits = Limits {
        array_elements: 2,
        ..Limits::default()
    };
    assert!(
        snapshot
            .invoke(vec![Value::String("abc".into())], limits)
            .is_err()
    );
    assert!(
        snapshot
            .invoke(vec![Value::String("é🇸🇪".into())], limits)
            .is_ok()
    );
    let limits = Limits {
        array_bytes: std::mem::size_of::<Value>() + 5,
        ..Limits::default()
    };
    assert!(
        snapshot
            .invoke(vec![Value::String("👨‍👩‍👧‍👦".into())], limits)
            .is_err()
    );
}
