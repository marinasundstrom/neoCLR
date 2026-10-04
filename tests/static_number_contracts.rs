use neoclr::{Limits, LoadedProgram, Value, assemble, assembler::parse_function_ref};

const SOURCE: &str = r#"
.module ContractProbe
.interface TestNumber<T>
.method static Add(T left, T right) -> T
.end
.end
.type TestInteger
.implements TestNumber<Int32>
.method static Add(Int32 left, Int32 right) -> Int32
ldarg left
ldarg right
add
ret
.end
.end
.function Run() -> Int32
ldc.i4 20
ldc.i4 22
call TestInteger::Add(Int32,Int32)
ret
.end
"#;

#[test]
fn static_interface_contract_requires_exact_concrete_member() {
    let module = assemble(SOURCE).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program
            .resolve_function(&parse_function_ref("Run()").unwrap())
            .unwrap()
            .invoke(vec![], Limits::default())
            .unwrap()
            .value,
        Value::Int32(42)
    );
    for changed in [
        SOURCE.replace(".method static Add(Int32", ".method instance Add(Int32"),
        SOURCE.replace(
            ".method static Add(Int32 left, Int32 right) -> Int32",
            ".method static Add(Int32 left, Int32 right) -> Int64",
        ),
        SOURCE.replace(
            ".method static Add(Int32",
            ".method private static Add(Int32",
        ),
    ] {
        let accepted =
            assemble(&changed).and_then(|module| LoadedProgram::new(&module).map(|_| ()));
        assert!(accepted.is_err(), "invalid conformance was admitted");
    }
}

#[test]
fn static_interface_virtual_defaults_and_direct_abstract_calls_are_rejected() {
    let body = SOURCE.replace(
        ".method static Add(T left, T right) -> T\n.end",
        ".method static virtual Add(T left, T right) -> T\nldarg left\nret\n.end",
    );
    assert!(assemble(&body).is_err());
    let direct = SOURCE.replace("call TestInteger::Add", "call TestNumber<Int32>::Add");
    let rejected = assemble(&direct).and_then(|module| {
        let program = LoadedProgram::new(&module)?;
        program.verify().map(|_| ())
    });
    assert!(rejected.is_err());
}

#[test]
fn numeric_implementation_matches_metadata_declaration_name() {
    let source = SOURCE
        .replace("TestInteger", "System.Int32")
        .replace(".module ContractProbe", ".module System");
    let mut module = assemble(&source).unwrap();
    let contract = module
        .functions
        .iter_mut()
        .find(|f| f.name == "TestNumber.Add")
        .unwrap();
    contract.name = "TestNumber.M_416464".into();
    contract.origin = Some(
        serde_json::from_value(serde_json::json!({
            "assembly": "Contracts", "module": "Contracts.dll", "name": "Add", "token": 0x06000001, "parameter_tokens": [0, 0]
        }))
        .unwrap(),
    );
    module.assemblies.push(serde_json::from_value(serde_json::json!({
        "name": "Contracts", "full_name": "Contracts", "modules": ["Contracts.dll"], "references": []
    })).unwrap());
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program
            .resolve_function(&parse_function_ref("Run()").unwrap())
            .unwrap()
            .invoke(vec![], Limits::default())
            .unwrap()
            .value,
        Value::Int32(42)
    );
    let implementation = module
        .functions
        .iter_mut()
        .find(|f| f.name == "System.Int32.Add")
        .unwrap();
    implementation.visibility = neoclr::metadata::Visibility::Private;
    assert!(
        LoadedProgram::new(&module).is_err(),
        "canonical lookup bypassed accessibility"
    );
}
