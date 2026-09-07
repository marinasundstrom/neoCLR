use neoclr::{
    Limits, LoadedProgram, Value, assemble,
    assembler::{assemble_modules, parse_function_ref, parse_type},
    metadata::Visibility,
};
const LIB: &str = ".module Models\n.revision r1\n.type internal Hidden\n.field Value Int32\n.method public static Number() -> Int32\nldc.i4 42\nret\n.end\n.end\n.function Build() -> Hidden\nldc.i4 42\nnewobj Hidden\nret\n.end\n.function Visible() -> Int32\ncall Hidden::Number()\nret\n.end";

#[test]
fn internal_type_sample_round_trips_and_executes_within_its_module() {
    let module = assemble(include_str!("../examples/type_visibility.neoil")).unwrap();
    assert_eq!(module.types[0].visibility, Visibility::Internal);
    let module = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    assert_eq!(program.run(Limits::default()).unwrap().output, ["42"]);
}

#[test]
fn foreign_type_references_are_denied_in_signatures_fields_properties_and_operands() {
    for extra in [
        ".function Bad(Hidden value) -> Void\nldvoid\nret\n.end",
        ".function Bad() -> Void\n.local Option<Hidden*> value\nldvoid\nret\n.end",
        ".type Other\n.field Stored [Models]Hidden\n.end",
        ".function Bad() -> Void\nldvoid\nret\nsizeof Hidden\nret\n.end",
        ".function Bad() -> Int32\ncall Hidden.Number()\nret\n.end",
        ".type Other\n.property static Value() -> Hidden\n.get Other::Read()\n.end\n.method static Read() -> Hidden\nfault \"unused\"\n.end\n.end",
    ] {
        let app = format!(".module App\n.references (Models#r1)\n{extra}");
        let fault = assemble_modules(&[&app, LIB]).unwrap_err();
        assert!(fault.message.contains("type access denied"), "{fault}");
    }
}

#[test]
fn implicit_return_types_and_field_types_cannot_bypass_foreign_access() {
    let app = ".module App\n.references (Models#r1)\n.function Main() -> Void\ncall Build()\npop\nldvoid\nret\n.end";
    assert!(
        assemble_modules(&[app, LIB])
            .unwrap_err()
            .message
            .contains("type access denied")
    );
    let library = format!(
        "{LIB}\n.type Outer\n.field Value Hidden\n.end\n.function OuterValue() -> Outer\ncall Build()\nnewobj Outer\nret\n.end"
    );
    let app = ".module App\n.references (Models#r1)\n.entry Main\n.function Main() -> Void\ncall OuterValue()\nldfld 0\npop\nldvoid\nret\n.end";
    let modules = assemble_modules(&[app, &library]).unwrap();
    let program = LoadedProgram::with_modules(
        &modules[0],
        neoclr::library::system().unwrap(),
        &modules[1..],
    )
    .unwrap();
    assert!(
        program
            .verify()
            .unwrap_err()
            .message
            .contains("type access denied")
    );
    let fault = program.run(Limits::default()).unwrap_err();
    assert!(fault.message.contains("type access denied"));
    assert!(fault.stack_trace.is_some());
}

#[test]
fn public_members_on_internal_types_are_not_host_invocation_capabilities() {
    let modules = assemble_modules(&[".module App\n.references (Models#r1)", LIB]).unwrap();
    let program = LoadedProgram::with_modules(
        &modules[0],
        neoclr::library::system().unwrap(),
        &modules[1..],
    )
    .unwrap();
    for target in ["Hidden::Number()", "Build()"] {
        assert!(
            program
                .resolve_function(&parse_function_ref(target).unwrap())
                .unwrap_err()
                .message
                .contains("type access denied")
        );
    }
    let visible = program
        .resolve_function(&parse_function_ref("Visible()").unwrap())
        .unwrap();
    assert_eq!(
        visible.invoke(vec![], Limits::default()).unwrap().value,
        Value::Int32(42)
    );
    assert!(
        program
            .resolve_type_identity(&parse_type("Hidden").unwrap())
            .is_ok()
    );
    assert!(
        program
            .layout_of(
                &parse_type("Hidden").unwrap(),
                neoclr::memory::TargetLayout::host()
            )
            .is_ok()
    );
    assert!(
        program
            .analyze_reachability(&[parse_function_ref("Hidden::Number()").unwrap()], 1)
            .is_ok()
    );
}

#[test]
fn generic_library_code_can_operate_on_a_caller_supplied_internal_type() {
    let library = ".module Containers\n.type Box<T>\n.field private Value T\n.method static Create(T value) -> Box<T>\nldarg value\nnewobj Box<T>\nret\n.end\n.method instance Read() -> T\nldarg this\nldfld 0\nret\n.end\n.end";
    let app = ".module App\n.references (Containers)\n.type internal Payload\n.field Value Int32\n.end\n.entry Main\n.function Main() -> Int32\nldc.i4 42\nnewobj Payload\ncall Box<Payload>::Create(Payload)\ncall instance Box<Payload>::Read()\nldfld 0\nret\n.end";
    let modules = assemble_modules(&[app, library]).unwrap();
    let program = LoadedProgram::with_modules(
        &modules[0],
        neoclr::library::system().unwrap(),
        &modules[1..],
    )
    .unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().value,
        Value::Int32(42)
    );
    assert!(
        program
            .resolve_function(&parse_function_ref("Box<Payload>::Create(Payload)").unwrap())
            .is_err()
    );
}

#[test]
fn top_level_private_is_rejected_and_legacy_types_stay_public() {
    let module = assemble(".module App\n.type Plain\n.end").unwrap();
    assert_eq!(module.types[0].visibility, Visibility::Public);
    let json = serde_json::to_string(&module).unwrap();
    assert!(!json.contains("visibility"));
    assert!(neoclr::load(&json).is_ok());
    for visibility in ["private", "protected", "unknown"] {
        let mut json = serde_json::to_value(&module).unwrap();
        json["types"][0]["visibility"] = visibility.into();
        assert!(neoclr::load(&json.to_string()).is_err());
    }
    assert!(assemble(".module App\n.type private Plain\n.end").is_err());
}
