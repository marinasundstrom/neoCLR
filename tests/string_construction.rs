use neoclr::{FaultCode, Limits, LoadedProgram, Module, RuntimeService, Value};
use std::{process::Command, sync::OnceLock};

fn library() -> &'static Module {
    static LIBRARY: OnceLock<Module> = OnceLock::new();
    LIBRARY.get_or_init(|| {
        let output = Command::new("python3").args(["-c",
            "import runpy; m=runpy.run_path('docs/experiments/raven-target/collection_library.py'); print(m['build'](m['ROOT'] / 'runtime/System.neoil'))"
        ]).output().unwrap();
        assert!(output.status.success());
        neoclr::assemble(std::str::from_utf8(&output.stdout).unwrap()).unwrap()
    })
}

fn program(body: &str, returns: &str) -> LoadedProgram {
    let library = library();
    let source = format!(
        ".module StringConstruction\n.entry Main\n.function Main() -> {returns}\n{body}\nret\n.end"
    );
    let module = neoclr::assembler::read_modules(
        &[neoclr::assembler::ModuleInput::Source(&source)],
        library,
    )
    .unwrap()
    .remove(0);
    let loaded = LoadedProgram::with_library(&module, library).unwrap();
    loaded.verify().unwrap();
    loaded
}

#[test]
fn character_array_construction_copies_full_graphemes_and_empty_text() {
    for texts in [
        vec![],
        vec!["F", "o", "o"],
        vec!["é", "👩‍💻", "\\u0000"],
        vec!["e", "́"],
    ] {
        let mut body = format!(
            ".local arrayref<Char> chars\n.local String result\nldc.i4 {}\nnewarr Char\nstloc chars\n",
            texts.len()
        );
        for (index, text) in texts.iter().enumerate() {
            body.push_str(&format!("ldloc chars\nldc.i4 {index}\nldstr \"{text}\"\ncall neoCLR.Runtime.CharFromString(String)\nstelem Char\n"));
        }
        body.push_str(
            "ldloc chars\ncall neoCLR.Runtime.StringFromChars(arrayref<Char>)\nstloc result\n",
        );
        if !texts.is_empty() {
            body.push_str("ldloc chars\nldc.i4 0\nldstr \"X\"\ncall neoCLR.Runtime.CharFromString(String)\nstelem Char\n");
        }
        body.push_str("ldloc result");
        let p = program(&body, "String");
        assert_eq!(
            p.run(Limits::default()).unwrap().value,
            Value::String(texts.join("").replace("\\u0000", "\0").into())
        );
        let graph = p
            .analyze_reachability(
                &[neoclr::assembler::parse_function_ref("Main()").unwrap()],
                16,
            )
            .unwrap();
        let uses = graph.required_services();
        assert!(uses.contains(&RuntimeService::StringOperations));
        assert!(uses.contains(&RuntimeService::ManagedArrays));
    }
}

#[test]
fn grapheme_index_is_not_a_utf8_byte_or_scalar_offset() {
    for (index, expected) in [(0, Some("é")), (1, Some("👩‍💻")), (-1, None), (2, None)] {
        let p = program(
            &format!(
                "ldstr \"é👩‍💻\"\nldc.i4 {index}\ncall neoCLR.Runtime.StringGraphemeAt(String,Int32)"
            ),
            "Char",
        );
        match expected {
            Some(text) => assert_eq!(
                p.run(Limits::default()).unwrap().value,
                Value::Char(text.into())
            ),
            None => assert_eq!(
                p.run(Limits::default()).unwrap_err().code,
                FaultCode::IndexOutOfRange
            ),
        }
    }
}

#[test]
fn explicit_string_count_requires_readonly_private_implementation() {
    for public in [false, true] {
        let mut module = library().clone();
        let method = module
            .functions
            .iter_mut()
            .find(|f| f.name == "System.String.CollectionCount")
            .unwrap();
        if public {
            method.visibility = neoclr::metadata::Visibility::Public;
        } else {
            method.receiver_readonly = false;
        }
        let fault = match LoadedProgram::new(&module) {
            Err(fault) => fault,
            Ok(_) => panic!("invalid explicit mapping accepted"),
        };
        assert!(
            fault.message.contains("explicit implementations require"),
            "{fault}"
        );
    }
}

#[test]
fn sequence_constructor_rejects_null() {
    let p = program(
        ".local System.Collections.Sequence<Char> missing
ldloca missing
initobj System.Collections.Sequence<Char>
ldloc missing
call neoCLR.Runtime.StringFromSequence(System.Collections.Sequence<Char>)",
        "String",
    );
    assert_eq!(
        p.run(Limits::default()).unwrap_err().code,
        FaultCode::NullReference
    );
}

#[test]
fn runtime_string_constructor_replaces_only_its_private_receiver() {
    let library = neoclr::assemble(".module System").unwrap();
    let source = r#"
.module NativeStringConstructor
.entry Main
.type System.String
.method instance .ctor(String value) -> void
ldarg value
starg this
ret
.end
.end
.function Main() -> String
ldstr "é👩‍💻"
newobj instance System.String::.ctor(String)
ret
.end
"#;
    let module = neoclr::assembler::read_modules(
        &[neoclr::assembler::ModuleInput::Source(source)],
        &library,
    )
    .unwrap()
    .remove(0);
    let program = LoadedProgram::with_library(&module, &library).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::String("é👩‍💻".into())
    );
}

#[test]
fn join_parts_validates_prefix_byte_count_and_preserves_snapshot() {
    for (count, bytes, expected) in [
        (0, 0, Some("")),
        (1, 1, Some("e")),
        (2, 6, Some("e•́")),
        (3, 3, None),
        (-1, 0, None),
        (2, 2, None),
        (2, 4, None),
        (2, 65537, None),
        (4, 3, None),
    ] {
        let source = format!(
            r#"
.module JoinParts
.entry Main
.function Main() -> String
.local arrayref<String> parts
.local String snapshot
ldc.i4 3
array.reserve String
stloc parts
ldloc parts
ldc.i4 0
ldstr "e"
stelem String
ldloc parts
ldc.i4 1
ldstr "́"
stelem String
ldloc parts
ldc.i4 {count}
ldstr "•"
ldc.i4 {bytes}
call neoCLR.Runtime.StringJoinParts(arrayref<String>,Int32,String,Int32)
stloc snapshot
ldloc parts
ldc.i4 0
ldstr "changed"
stelem String
ldloc snapshot
ret
.end
"#
        );
        let library = neoclr::assemble(concat!(
            ".module System\n",
            include_str!("../runtime/neoCLR/Runtime/StringJoinParts.neoil")
        ))
        .unwrap();
        let module = neoclr::assemble(&source).unwrap();
        let p = LoadedProgram::with_library(&module, &library).unwrap();
        p.verify().unwrap();
        match expected {
            Some(text) => assert_eq!(
                p.run(Limits::default()).unwrap().value,
                Value::String(text.into())
            ),
            None => assert_eq!(
                p.run(Limits::default()).unwrap_err().code,
                FaultCode::RuntimeError
            ),
        }
    }
}
