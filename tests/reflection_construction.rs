use neoclr::assembler::parse_function_ref;
use neoclr::{FaultCode, Limits, LoadedProgram, Value, assemble, run};

const SERVICES: &str = r#"
.function neoCLR.Runtime.ReflectionConstructionCheck(System.RuntimeTypeHandle) -> Int32
.methodimpl InternalCall
.end
.function neoCLR.Runtime.ReflectionConstruct(System.RuntimeTypeHandle) -> System.Object
.methodimpl InternalCall
.end
"#;
const OBJECT: &str = r#"
.type class abstract System.Object
.method instance .ctor() -> noresult
ret
.end
.end
"#;
const MODEL: &str = r#"
.type class Model
.extends System.Object
.field Number Int32
.method instance .ctor() -> noresult
ldarg this
call instance System.Object::.ctor()
ldarg this
ldc.i4 42
stfld Model::Number
ret
.end
.end
"#;
fn module(types: &str, body: &str) -> neoclr::Module {
    assemble(&format!(
        ".module ReflectionConstruction\n.entry Main\n{OBJECT}{SERVICES}{types}\n.function Main() -> Int32\n{body}\nret\n.end"
    )).unwrap()
}
const CREATE: &str = "ldtoken Model\ncall neoCLR.Runtime.ReflectionConstruct(System.RuntimeTypeHandle)\ncastclass Model\nldfld Model::Number";

#[test]
fn executes_real_constructor_and_round_trips_artifact() {
    let module = module(MODEL, CREATE);
    let restored = neoclr::load(&serde_json::to_string(&module).unwrap()).unwrap();
    assert_eq!(
        run(&restored, Limits::default()).unwrap().value,
        Value::Int32(42)
    );
}

#[test]
fn rejects_unsupported_types_and_nonpublic_or_absent_constructors() {
    for (types, name, expected) in [
        (MODEL.to_string(), "Model", 0),
        (
            MODEL.replace(
                ".method instance .ctor()",
                ".method private instance .ctor()",
            ),
            "Model",
            3,
        ),
        (
            MODEL.replace(".type class Model", ".type internal class Model"),
            "Model",
            3,
        ),
        (
            ".type class Model\n.extends System.Object\n.end\n".into(),
            "Model",
            4,
        ),
        (
            ".type class abstract Model\n.extends System.Object\n.end\n".into(),
            "Model",
            2,
        ),
        (".type Model\n.end\n".into(), "Model", 2),
        (
            ".type class Model<T>\n.extends System.Object\n.end\n".into(),
            "Model<Int32>",
            2,
        ),
    ] {
        let module = module(
            &types,
            &format!(
                "ldtoken {name}\ncall neoCLR.Runtime.ReflectionConstructionCheck(System.RuntimeTypeHandle)"
            ),
        );
        assert_eq!(
            run(&module, Limits::default()).unwrap().value,
            Value::Int32(expected),
            "{types}"
        );
        if expected != 0 {
            let rejected = module_with_create(&types, name);
            assert!(
                run(&rejected, Limits::default())
                    .unwrap_err()
                    .message
                    .contains("reflection construction rejected")
            );
        }
    }
}
fn module_with_create(types: &str, name: &str) -> neoclr::Module {
    module(
        types,
        &format!(
            "ldtoken {name}\ncall neoCLR.Runtime.ReflectionConstruct(System.RuntimeTypeHandle)\npop\nldc.i4 0"
        ),
    )
}

#[test]
fn host_boundary_does_not_expose_the_private_type_handle_service() {
    let types = format!(
        "{MODEL}\n.function Probe(System.RuntimeTypeHandle handle) -> Int32\nldarg handle\ncall neoCLR.Runtime.ReflectionConstructionCheck(System.RuntimeTypeHandle)\nret\n.end"
    );
    let module = module(&types, "ldc.i4 0");
    let program = LoadedProgram::new(&module).unwrap();
    let error = match program
        .resolve_function(&parse_function_ref("Probe(System.RuntimeTypeHandle)").unwrap())
    {
        Ok(_) => panic!("private type-handle service exposed to host invocation"),
        Err(error) => error,
    };
    assert!(
        error
            .message
            .contains("runtime type handles cannot be imported from the host")
    );
}

#[test]
fn constructor_faults_and_frame_limits_are_not_hidden() {
    let module = module(MODEL, CREATE);
    let error = run(
        &module,
        Limits {
            frames: 2,
            ..Limits::default()
        },
    )
    .unwrap_err();
    assert_eq!(error.code, FaultCode::StackOverflow);
    let broken = MODEL.replace("ldc.i4 42", "ldc.i4 1\nldc.i4 0\ndiv");
    let faulting = module_with_create(&broken, "Model");
    assert!(
        run(&faulting, Limits::default())
            .unwrap_err()
            .message
            .contains("zero")
    );
}

#[test]
fn construction_retains_receiver_during_collection() {
    let churn = "ldc.i4 1\nnewobj Garbage\npop\n".repeat(24);
    let types = format!(
        "{}\n.type class Garbage\n.field N Int32\n.end\n",
        MODEL.replace(
            "ldarg this\nldc.i4 42",
            &format!("{churn}ldarg this\nldc.i4 42")
        )
    );
    let module = module(&types, CREATE);
    let result = run(
        &module,
        Limits {
            heap_objects: 4,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert!(result.heap.collections() > 1);
    assert_eq!(result.heap.len(), 0);
}

#[test]
fn binding_requires_exact_signature() {
    for signature in [
        "ReflectionConstruct(Int32) -> System.Object",
        "ReflectionConstruct(System.RuntimeTypeHandle) -> Int32",
        "ReflectionConstructionCheck(System.RuntimeTypeHandle) -> Boolean",
    ] {
        assert!(assemble(&format!(".module Invalid\n{OBJECT}\n.function neoCLR.Runtime.{signature}\n.methodimpl InternalCall\n.end")).is_err());
    }
}

#[test]
fn reachability_reports_dynamic_execution_and_allocation_services() {
    let module = module(MODEL, CREATE);
    let program = LoadedProgram::new(&module).unwrap();
    let graph = program
        .analyze_reachability(&[parse_function_ref("Main()").unwrap()], 8)
        .unwrap();
    let services = graph.required_services();
    for expected in [
        neoclr::RuntimeService::ReflectionExecution,
        neoclr::RuntimeService::TypeInspection,
        neoclr::RuntimeService::FrameAllocation,
        neoclr::RuntimeService::ManagedHeap,
    ] {
        assert!(services.contains(&expected));
    }
}

#[test]
fn imported_construction_requires_explicit_public_source_access() {
    for (type_access, method_access, expected) in [
        (
            ",\"publicly_visible\":true",
            ",\"member_access\":\"Public\"",
            0,
        ),
        (
            ",\"publicly_visible\":false",
            ",\"member_access\":\"Public\"",
            3,
        ),
        (
            ",\"publicly_visible\":true",
            ",\"member_access\":\"Private\"",
            3,
        ),
        (",\"publicly_visible\":true", "", 3),
        ("", ",\"member_access\":\"Public\"", 3),
    ] {
        let types = MODEL.replace(".type class Model\n", &format!(
            ".type class Model\n.origin {{\"assembly\":\"Fixture\",\"module\":\"Fixture\",\"name\":\"Model\",\"token\":33554433,\"field_tokens\":[67108865]{type_access}}}\n"
        )).replace(".method instance .ctor() -> noresult\n", &format!(
            ".method instance .ctor() -> noresult\n.origin {{\"assembly\":\"Fixture\",\"module\":\"Fixture\",\"name\":\".ctor\",\"token\":100663297{method_access}}}\n"
        ));
        let types = format!(
            ".assembly {{\"name\":\"Fixture\",\"full_name\":\"Fixture\",\"modules\":[\"Fixture\"],\"references\":[]}}\n{types}"
        );
        let candidate = module(
            &types,
            "ldtoken Model\ncall neoCLR.Runtime.ReflectionConstructionCheck(System.RuntimeTypeHandle)",
        );
        assert_eq!(
            run(&candidate, Limits::default()).unwrap().value,
            Value::Int32(expected)
        );
    }
}
