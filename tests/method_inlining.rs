use neoclr::{Limits, Value, assemble, run};

#[test]
fn implementation_options_preserve_interpreter_results_and_faults() {
    for flags in [
        "NoInlining",
        "AggressiveInlining",
        "NoInlining|AggressiveInlining",
    ] {
        let source = format!(
            ".module InlineFlags\n.references ()\n.entry Main\n.function Main() -> Int32\n.methodimpl {flags}\nldc.i4 7\ncall Compute(Int32)\nret\n.end\n.function Compute(Int32 value) -> Int32\n.methodimpl {flags}\nldarg 0\nldc.i4 3\nmul\nret\n.end\n"
        );
        let module = assemble(&source).unwrap();
        assert_eq!(
            run(&module, Limits::default()).unwrap().value,
            Value::Int32(21)
        );
        let failing = assemble(&source.replace("ldc.i4 3\nmul", "ldc.i4 0\ndiv")).unwrap();
        let error = run(&failing, Limits::default()).unwrap_err();
        assert!(error.to_string().contains("zero"));
    }
}
