use neoclr::{Limits, LoadedProgram, Value, assemble};

fn program(service: &str, element: &str) -> Result<LoadedProgram, neoclr::Fault> {
    let module = assemble(&format!(r#".module System
.function neoCLR.Runtime.{service}(String text) -> arrayref<{element}>
.methodimpl InternalCall
.end
.function Main() -> Int32
ldstr "é🙂"
call neoCLR.Runtime.{service}(String)
ldlen
conv.i4
ret
.end
"#))?;
    LoadedProgram::new(&module)
}

#[test]
fn managed_text_vectors_match_exact_elements_and_enforce_limits() {
    for (service, element, count) in [
        ("StringGraphemes", "Char", 2),
        ("StringScalars", "UInt32", 2),
        ("Utf8Encode", "Byte", 6),
    ] {
        let p = program(service, element).unwrap();
        assert_eq!(p.resolve_function(&neoclr::assembler::parse_function_ref("Main()").unwrap()).unwrap().invoke(vec![], Limits::default()).unwrap().value, Value::Int32(count));
        assert!(p.resolve_function(&neoclr::assembler::parse_function_ref("Main()").unwrap()).unwrap().invoke(vec![], Limits { array_elements: 1, ..Limits::default() }).is_err());
        assert!(p.resolve_function(&neoclr::assembler::parse_function_ref("Main()").unwrap()).unwrap().invoke(vec![], Limits { heap_objects: 0, ..Limits::default() }).is_err());
        for wrong in ["String", "Int32"] {
            assert!(program(service, wrong).is_err());
        }
    }
}
