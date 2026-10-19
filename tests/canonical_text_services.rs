use neoclr::{
    Limits, LoadedProgram, RuntimeService, Value, assemble, assembler::parse_function_ref,
};

fn seed() -> neoclr::Module {
    let mut source = ".module System\n.references ()\n".to_owned();
    for kind in ["String", "Char"] {
        for (operation, arguments) in [
            ("Compare", format!("{kind}, {kind}")),
            ("Hash", kind.into()),
        ] {
            source += &format!(
                ".function neoCLR.Runtime.{kind}{operation}Canonical({arguments}) -> Int32\n.methodimpl InternalCall\n.end\n"
            );
        }
    }
    assemble(&source).unwrap()
}

#[test]
fn canonical_services_preserve_text_and_share_char_hashes() {
    let library = seed();
    let mut wrappers = String::from(".module App\n");
    for kind in ["String", "Char"] {
        wrappers += &format!(
            ".function {kind}Compare({kind} left,{kind} right) -> Int32\nldarg left\nldarg right\ncall neoCLR.Runtime.{kind}CompareCanonical({kind},{kind})\nret\n.end\n"
        );
        wrappers += &format!(
            ".function {kind}Hash({kind} value) -> Int32\nldarg value\ncall neoCLR.Runtime.{kind}HashCanonical({kind})\nret\n.end\n"
        );
    }
    let app = neoclr::assembler::read_modules(
        &[neoclr::assembler::ModuleInput::Source(&wrappers)],
        &library,
    )
    .unwrap()
    .remove(0);
    let program = LoadedProgram::with_library(&app, &library).unwrap();
    for (left, right, expected) in [
        ("é", "e\u{301}", 0),
        ("각", "각", 0),
        ("ﬁ", "f", 1),
        ("A", "a", -1),
    ] {
        let mut hashes = Vec::new();
        for kind in ["String", "Char"] {
            let wrap = |s: &str| {
                if kind == "String" {
                    Value::String(s.into())
                } else {
                    Value::Char(s.into())
                }
            };
            let signature = parse_function_ref(&format!("{kind}Compare({kind},{kind})")).unwrap();
            let compare = program.resolve_function(&signature).unwrap();
            assert_eq!(
                compare
                    .invoke(vec![wrap(left), wrap(right)], Limits::default())
                    .unwrap()
                    .value,
                Value::Int32(expected)
            );
            let signature = parse_function_ref(&format!("{kind}Hash({kind})")).unwrap();
            let hash = program.resolve_function(&signature).unwrap();
            let values: Vec<_> = [left, right]
                .into_iter()
                .map(|s| hash.invoke(vec![wrap(s)], Limits::default()).unwrap().value)
                .collect();
            if expected == 0 {
                assert_eq!(values[0], values[1]);
            }
            hashes.push(values);
        }
        assert_eq!(hashes[0], hashes[1]);
    }
    let root = parse_function_ref("StringCompare(String,String)").unwrap();
    let graph = program.analyze_reachability(&[root], 16).unwrap();
    assert!(
        graph
            .required_services()
            .contains(&RuntimeService::StringOperations)
    );
}

#[test]
fn canonical_services_reject_wrong_return_types() {
    let app = assemble(".module App").unwrap();
    for index in 0..4 {
        let mut library = seed();
        library.functions[index].returns = neoclr::metadata::Type::Char;
        assert!(
            LoadedProgram::with_library(&app, &library)
                .unwrap_err()
                .message
                .contains("runtime binding return type mismatch")
        );
    }
}
