use neoclr::{Limits, LoadedProgram, Module, Value};
use std::{process::Command, sync::OnceLock};
fn library() -> &'static Module {
    static LIBRARY: OnceLock<Module> = OnceLock::new();
    LIBRARY.get_or_init(|| {
        let output = Command::new("python3").current_dir(env!("CARGO_MANIFEST_DIR"))
            .args(["-c", "import runpy; m=runpy.run_path('docs/experiments/raven-target/collection_library.py'); print(m['build'](m['ROOT'] / 'runtime/System.neoil'))"])
            .output().unwrap();
        assert!(output.status.success());
        neoclr::assemble(std::str::from_utf8(&output.stdout).unwrap()).unwrap()
    })
}

// The selected Raven profile has interface contracts; the bundled legacy profile
// still has value descriptors, so validate callers against the selected library.
fn assemble(source: &str) -> Result<Module, neoclr::Fault> {
    neoclr::assembler::read_modules(&[neoclr::assembler::ModuleInput::Source(source)], library())
        .map(|mut modules| modules.remove(0))
}

fn run(body: &str, declarations: &str) -> Result<neoclr::Execution, neoclr::Fault> {
    let app = assemble(&format!(
        ".module Display\n.entry Main\n{declarations}\n.function Main() -> String\n{body}\nret\n.end"
    ))?;
    let program = LoadedProgram::with_library(&app, library())?;
    program.verify()?;
    program.run(Limits::default())
}

#[test]
fn object_virtual_display_selects_override_and_explicit_base_call() {
    let declarations = ".type class Named\n.extends System.Object\n.method instance override ToString() -> String\nldstr \"named\"\nret\n.end\n.end";
    for (opcode, expected) in [("callvirt", "named"), ("call", "Named")] {
        let result = run(
            &format!("newobj Named\n{opcode} instance System.Object::ToString()"),
            declarations,
        )
        .unwrap();
        assert_eq!(result.value, Value::String(expected.into()));
    }
}

#[test]
fn rootless_class_and_array_use_default_object_slot() {
    for (body, declaration, expected) in [
        ("newobj Plain", ".type class Plain\n.end", "Plain"),
        (
            "ldc.i4 0\nnewarr Int32\ncastclass System.Object",
            "",
            "arrayref<System.Int32>",
        ),
    ] {
        let result = run(
            &format!("{body}\ncallvirt instance System.Object::ToString()"),
            declaration,
        )
        .unwrap();
        assert_eq!(result.value, Value::String(expected.into()));
    }
}

#[test]
fn null_object_display_reports_null_reference() {
    let fault = run(".local System.Object value\nldloca value\ninitobj System.Object\nldloc value\ncallvirt instance System.Object::ToString()", "").unwrap_err();
    assert_eq!(fault.code, neoclr::FaultCode::NullReference);
}

#[test]
fn boxed_value_and_string_virtual_display_are_not_silently_type_names() {
    for body in [
        "ldc.i4 42\nbox Int32",
        "ldstr \"hello\"\ncastclass System.Object",
    ] {
        let fault = run(
            &format!("{body}\ncallvirt instance System.Object::ToString()"),
            "",
        )
        .unwrap_err();
        assert_ne!(fault.code, neoclr::FaultCode::NullReference);
    }
}

#[test]
fn abstract_object_cannot_be_constructed_even_in_raw_il() {
    let fault = run(
        "newobj instance System.Object::.ctor()\ncallvirt instance System.Object::ToString()",
        "",
    )
    .unwrap_err();
    assert!(fault.message.contains("abstract"), "{fault}");
}

#[test]
fn rootless_object_default_remains_reachable_with_an_abstract_root() {
    let app = assemble(".module Display\n.entry Main\n.type class Plain\n.end\n.function Main() -> String\nnewobj Plain\ncallvirt instance System.Object::ToString()\nret\n.end").unwrap();
    let program = LoadedProgram::with_library(&app, library()).unwrap();
    let graph = program
        .analyze_reachability(
            &[neoclr::assembler::parse_function_ref("Main()").unwrap()],
            1000,
        )
        .unwrap();
    assert!(
        graph
            .functions
            .iter()
            .any(|function| function.target.name == "System.Object.ToString")
    );
}

#[test]
fn nonvirtual_class_callvirt_has_one_reachable_implementation() {
    let app = neoclr::assemble(".module Calls\n.entry Main\n.type class Base\n.method instance Read() -> Int32\nldc.i4 42\nret\n.end\n.end\n.type class Child\n.extends Base\n.end\n.function Main() -> Int32\nnewobj Child\ncallvirt instance Base::Read()\nret\n.end").unwrap();
    let program = LoadedProgram::new(&app).unwrap();
    let graph = program.analyze_reachability(
        &[neoclr::assembler::parse_function_ref("Main()").unwrap()], 10
    ).unwrap();
    assert_eq!(graph.functions.len(), 2);
    assert!(graph.functions.iter().any(|f| f.target.name == "Base.Read"));
}
