use neoclr::{Limits, assemble, run, verify};

fn program(body: &str) -> neoclr::Module {
    assemble(&format!(
        ".module Test\n.entry Main\n.function Main() -> Int32\n{body}\n.end"
    ))
    .unwrap()
}

#[test]
fn verifier_accepts_all_samples_without_executing_them() {
    for file in std::fs::read_dir("examples").unwrap() {
        let path = file.unwrap().path();
        if path
            .extension()
            .is_some_and(|extension| extension == "neoil")
        {
            let module = assemble(&std::fs::read_to_string(&path).unwrap()).unwrap();
            verify(&module).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        }
    }
}

#[test]
fn reports_maximum_stack_and_reachable_instructions() {
    let module = program("ldc.i4 1\ndup\nldc.i4 2\nmul\nadd\nret\npop");
    let report = verify(&module).unwrap();
    let function = &report.functions[0];
    assert_eq!(function.maximum_stack, 3);
    assert_eq!(function.reachable_instructions, 6);
    assert_eq!(function.function_index, 0);
    assert_eq!(function.name, "Main");
    assert!(verify(neoclr::library::system().unwrap()).is_ok());
}

#[test]
fn rejects_underflow_bad_returns_and_reachable_fallthrough() {
    for (body, message) in [
        ("add\nret", "underflow"),
        ("ret", "underflow"),
        ("ldc.i4 1\nldc.i4 2\nret", "exactly one"),
        ("ldc.i4 1", "fallthrough"),
        (
            "ldc.i4 1\ncall System.Console::WriteLine(Int32)\nldc.i4 2\nret",
            "exactly one",
        ),
    ] {
        let fault = verify(&program(body)).unwrap_err();
        assert!(fault.message.contains(message), "{body}: {fault}");
        assert_eq!(fault.function.as_deref(), Some("Main"));
        assert!(fault.instruction.is_some());
    }
}

#[test]
fn branches_switches_and_loops_require_consistent_stack_heights() {
    let body = "ldc.i4 0\nbrtrue Empty\nldc.i4 42\nbr Join\nEmpty:\nbr Join\nJoin:\nfault \"stop\"";
    assert!(
        verify(&program(body))
            .unwrap_err()
            .message
            .contains("stack heights")
    );
    let loop_body = "Again:\nldc.i4 1\nbr Again";
    assert!(
        verify(&program(loop_body))
            .unwrap_err()
            .message
            .contains("stack heights")
    );
    let body = "ldc.i4 0\nswitch (Case, Case)\nldc.i4 1\nbr Done\nCase:\nldc.i4 2\nDone:\nret";
    assert_eq!(
        verify(&program(body)).unwrap().functions[0].maximum_stack,
        1
    );
    assert!(verify(&program("Again:\nbr Again")).is_ok());
}

#[test]
fn local_initialization_is_intersected_at_joins_and_loop_headers() {
    let body = ".local Int32 value\nldc.i4 0\nbrtrue Missing\nldc.i4 42\nstloc value\nbr Done\nMissing:\nbr Done\nDone:\nldloc value\nret";
    assert!(
        verify(&program(body))
            .unwrap_err()
            .message
            .contains("not initialized")
    );
    let late = ".local Int32 value\nldc.i4 1\nbrtrue Init\nbr Slow1\nInit:\nldc.i4 42\nstloc value\nbr Join\nSlow1:\nbr Slow2\nSlow2:\nbr Slow3\nSlow3:\nbr Join\nJoin:\nldloc value\nret";
    assert!(
        verify(&program(late))
            .unwrap_err()
            .message
            .contains("not initialized")
    );
    let both = body.replace(
        "Missing:\nbr Done",
        "Missing:\nldc.i4 7\nstloc value\nbr Done",
    );
    assert!(verify(&program(&both)).is_ok());
    let body = ".local Int32 value\nAgain:\nldloc value\npop\nldc.i4 42\nstloc value\nbr Again";
    assert!(verify(&program(body)).is_err());
    let body = ".local Int32 value\nldc.i4 42\nstloc value\nAgain:\nldloc value\nbrtrue Again\nldloc value\nret";
    assert!(verify(&program(body)).is_ok());
}

#[test]
fn validates_unused_methods_and_symbolic_generic_stack_effects() {
    let source = ".module Test\n.type Box<T>\n.field Value T\n.method static Make(T value) -> Box<T>\nldarg value\nnewobj Box<T>\nret\n.end\n.end";
    assert_eq!(
        verify(&assemble(source).unwrap()).unwrap().functions[0].maximum_stack,
        1
    );
    assert!(verify(&assemble(&source.replace("ldarg value\n", "")).unwrap()).is_err());
    let source = ".module Test\n.type Empty<T>\n.end\n.function F() -> Empty<Void>\nnewobj Empty<Void>\nret\n.end";
    assert_eq!(
        verify(&assemble(source).unwrap()).unwrap().functions[0].maximum_stack,
        1
    );
}

#[test]
fn structural_validation_precedes_analysis_and_fault_is_terminal() {
    let mut module = program("ldc.i4 1\nret");
    module.functions[0].body[0] = neoclr::metadata::Instruction::Branch(999);
    assert!(verify(&module).is_err());
    assert!(verify(&program("fault \"stop\"\npop")).is_ok());
    let source =
        ".module Test\n.function F(Int32* p) -> Int32\nldarg p\nunaligned. 1\nldind.i4\nret\n.end";
    assert_eq!(
        verify(&assemble(source).unwrap()).unwrap().functions[0].maximum_stack,
        1
    );
    assert!(verify(&assemble(&source.replace("ldarg p\n", "")).unwrap()).is_err());
}

#[test]
fn verifier_is_explicit_and_does_not_claim_type_safety() {
    let invalid = program("ldstr \"wrong return type\"\nret");
    assert!(verify(&invalid).is_ok());
    assert!(run(&invalid, Limits::default()).is_err());
    let uninitialized = program(".local Int32 value\nldloc value\nret");
    assert!(verify(&uninitialized).is_err());
    // Assembling still supports runtime-diagnostic fixtures without opting into verification.
    assert!(run(&uninitialized, Limits::default()).is_err());
}

#[test]
fn cli_verify_accepts_source_and_serialized_metadata_without_execution() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_neoclr"))
        .args(["verify", "examples/fault.neoil"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("types checked at runtime")
    );
    let path = std::env::temp_dir().join(format!("neoclr-verify-{}.neo.json", std::process::id()));
    let module = program("pop\nret");
    std::fs::write(&path, serde_json::to_string(&module).unwrap()).unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_neoclr"))
        .arg("verify")
        .arg(&path)
        .output()
        .unwrap();
    std::fs::remove_file(path).unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("at Main:0")
    );
}
