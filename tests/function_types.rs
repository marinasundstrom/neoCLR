use neoclr::assembler::parse_type;
use neoclr::metadata::Type;
use neoclr::{Limits, LoadedProgram, Value, assemble, run, verify};

fn module(body: &str, declarations: &str) -> neoclr::Module {
    assemble(&format!(".module Structural\n.entry Main\n{declarations}\n.function Main() -> Int32\n{body}\nret\n.end")).unwrap()
}

#[test]
fn structural_call_requires_no_nominal_declaration() {
    let m = module(
        "function.bind fn<Int32,Int32> = Identity<Int32>(Int32)\nldc.i4 42\ncall instance fn<Int32,Int32>::Invoke(Int32)",
        ".function Identity<T>(T) -> T\nldarg 0\nret\n.end",
    );
    verify(&m).unwrap();
    let json = serde_json::to_string(&m).unwrap();
    assert!(json.contains("function.bind"));
    let m = neoclr::load(&json).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(42));
    let program = LoadedProgram::new(&m).unwrap();
    let shape = parse_type("fn<Int32,Int32>").unwrap();
    let identity = program.resolve_type_identity(&shape).unwrap();
    assert!(matches!(identity, neoclr::TypeIdentity::Function { .. }));
}

#[test]
fn structural_shapes_preserve_parameter_modes_and_generic_substitution() {
    let open = parse_type("fn<readonly !0&,out !0&,!0>").unwrap();
    assert_eq!(
        open.substitute_type_parameters(&[Type::Int32]).unwrap(),
        parse_type("fn<readonly Int32&,out Int32&,Int32>").unwrap()
    );
    assert_ne!(
        parse_type("fn<Int32&,Void>").unwrap(),
        parse_type("fn<out Int32&,Void>").unwrap()
    );
    assert_ne!(
        parse_type("fn<Void>").unwrap(),
        parse_type("fn<noresult Void>").unwrap()
    );
    assert!(parse_type("fn<out Int32,Void>").is_err());
    assert!(parse_type("fn<outtrue Int32&,Int32>").is_err());
}

#[test]
fn incompatible_structural_binding_is_rejected() {
    let error = assemble(".module Bad\n.function Wrong(Int32) -> Boolean\nldc.bool true\nret\n.end\n.function Main() -> Void\nfunction.bind fn<Int32,Int32> = Wrong(Int32)\npop\nldvoid\nret\n.end").unwrap_err();
    assert!(error.message.contains("signature"), "{}", error.message);
}

const COUNTER: &str = ".type Counter\n.field Value Int32\n.method instance byref Add(Int32) -> Int32\nldarg 0\nldflda Counter::Value\nldarg 0\nldfld Counter::Value\nldarg 1\nadd\nstobj Int32\nldarg 0\nldfld Counter::Value\nret\n.end\n.end";

#[test]
fn function_object_retains_shared_receiver_under_collection() {
    let m = module(
        ".local fn<Int32,Int32> callback\nldc.i4 40\nnewobj Counter\nheap.new\nfunction.bind fn<Int32,Int32> = instance Counter::Add(Int32)\nstloc callback\nldc.i4 100\nnewobj Counter\nheap.new\npop\nldc.i4 200\nnewobj Counter\nheap.new\npop\nldloc callback\nldc.i4 2\ncall instance fn<Int32,Int32>::Invoke(Int32)",
        COUNTER,
    );
    verify(&m).unwrap();
    let result = run(
        &m,
        Limits {
            heap_objects: 2,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert!(result.heap.statistics().collections > 0);
}

#[test]
fn structural_binding_still_rejects_frame_receivers() {
    let m = module(
        ".local Counter value\nldc.i4 40\nnewobj Counter\nstloc value\nldloca value\nfunction.bind fn<Int32,Int32> = instance Counter::Add(Int32)\nldc.i4 2\ncall instance fn<Int32,Int32>::Invoke(Int32)",
        COUNTER,
    );
    assert!(
        run(&m, Limits::default())
            .unwrap_err()
            .message
            .contains("frame-backed")
    );
}

#[test]
fn functions_with_identical_shapes_share_a_storage_contract() {
    let m = module(
        ".local fn<Int32,Int32> callback\nfunction.bind fn<Int32,Int32> = First(Int32)\nstloc callback\nfunction.bind fn<Int32,Int32> = Second(Int32)\nstloc callback\nldloc callback\nldc.i4 40\ncall instance fn<Int32,Int32>::Invoke(Int32)",
        ".function First(Int32) -> Int32\nldarg 0\nret\n.end\n.function Second(Int32) -> Int32\nldarg 0\nldc.i4 2\nadd\nret\n.end",
    );
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(42));
}

#[test]
fn no_result_invoke_preserves_the_caller_stack() {
    for instruction in ["call", "callvirt"] {
        let m = module(
            &format!(
                "ldc.i4 42\nfunction.bind fn<noresult Void> = Empty()\n{instruction} instance fn<noresult Void>::Invoke()"
            ),
            ".function Empty() -> noresult\nret\n.end",
        );
        verify(&m).unwrap();
        assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(42));
    }
}

#[test]
fn higher_order_generic_functions_accept_structural_shapes() {
    let m = module(
        "function.bind fn<Int32,Int32> = Identity<Int32>(Int32)\ncall Keep<fn<Int32,Int32>>(fn<Int32,Int32>)\nldc.i4 42\ncallvirt instance fn<Int32,Int32>::Invoke(Int32)",
        ".function Keep<T>(T) -> T\nldarg 0\nret\n.end\n.function Identity<T>(T) -> T\nldarg 0\nret\n.end",
    );
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(42));
    let graph = LoadedProgram::new(&m)
        .unwrap()
        .analyze_reachability(
            &[neoclr::assembler::parse_function_ref("Main()").unwrap()],
            100,
        )
        .unwrap();
    let main = &graph.functions[graph.roots[0]];
    assert_eq!(main.bindings.len(), 1);
    assert_eq!(main.function_invocations.len(), 1);
}

#[test]
fn nominal_classification_follows_identity_not_display_names() {
    for (shape, expected) in [
        ("Int32", 1),
        ("fn<Int32,Int32>", 0),
        ("Int32&", 0),
        ("Int32[]", 0),
    ] {
        let m = module(
            &format!(
                "ldtoken {shape}\nldc.i4 12\ncall neoCLR.Runtime.TypeShape(System.RuntimeTypeHandle,Int32)\nbrtrue Nominal\nldc.i4 0\nret\nNominal:\nldc.i4 1"
            ),
            "",
        );
        assert_eq!(
            run(&m, Limits::default()).unwrap().value,
            Value::Int32(expected)
        );
    }
}

#[test]
fn named_generic_parameters_bind_inside_function_shapes() {
    let m = module(
        "function.bind fn<Int32,Int32> = Identity<Int32>(Int32)\nldc.i4 42\ncall Apply<Int32>(fn<Int32,Int32>,Int32)",
        ".type Holder<T>\n.field Callback fn<T,T>\n.end\n.function Identity<T>(T) -> T\nldarg 0\nret\n.end\n.function Apply<T>(fn<T,T> callback,T value) -> T\nldarg callback\nldarg value\ncall instance fn<T,T>::Invoke(T)\nret\n.end",
    );
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(42));
    let field = &m
        .types
        .iter()
        .find(|ty| ty.name == "Holder")
        .unwrap()
        .fields[0];
    assert_eq!(field.ty, parse_type("fn<!0,!0>").unwrap());
}

#[test]
fn legacy_delegate_artifacts_and_source_declarations_are_rejected() {
    assert!(
        assemble(".module Legacy\n.delegate Old\n.method instance Invoke() -> Int32\n.end\n.end")
            .is_err()
    );
    assert!(
        neoclr::frontend::compile("delegate Old() -> int\nfunc Main() -> int { return 0 }")
            .unwrap_err()
            .message
            .contains("removed")
    );
    let m = module(
        "function.bind fn<Int32,Int32> = Identity(Int32)\nldc.i4 42\ncall instance fn<Int32,Int32>::Invoke(Int32)",
        ".function Identity(Int32) -> Int32\nldarg 0\nret\n.end",
    );
    let json = serde_json::to_string(&m).unwrap();
    assert!(neoclr::load(&json.replace("function.bind", "delegate.bind")).is_err());
    assert!(neoclr::load(&json.replace("function_type", "delegate")).is_err());
    let nominal = assemble(".module Legacy\n.type Old\n.end").unwrap();
    let mut json = serde_json::to_value(nominal).unwrap();
    json["types"][0]["representation"] = "Delegate".into();
    assert!(neoclr::load(&json.to_string()).is_err());
    assert!(assemble(".module Legacy\n.function Main() -> Int32\ndelegate.bind fn<Int32,Int32> = Identity(Int32)\nldc.i4 42\nret\n.end").is_err());
}

#[test]
fn function_identity_uses_loaded_component_identities() {
    let modules = neoclr::assembler::assemble_modules(&[
        ".module App\n.references (Left,Right)\n.entry Main\n.function Main() -> Int32\nldc.i4 0\nret\n.end",
        ".module Left\n.type Left.Payload\n.field Number Int32\n.end",
        ".module Right\n.type Right.Payload\n.field Number Int32\n.end",
    ]).unwrap();
    let program = LoadedProgram::with_modules(
        &modules[0],
        neoclr::library::system().unwrap(),
        &modules[1..],
    )
    .unwrap();
    let identity = |text| {
        program
            .resolve_type_identity(&parse_type(text).unwrap())
            .unwrap()
    };
    assert_eq!(
        identity("fn<[System]Int32,[System]Int32>"),
        identity("fn<Int32,Int32>")
    );
    assert_ne!(
        identity("fn<[Left]Left.Payload,Int32>"),
        identity("fn<[Right]Right.Payload,Int32>")
    );
}

#[test]
fn artifacts_cannot_redefine_the_synthesized_invoke_contract() {
    assert!(assemble(".module Bad\n.type class NamedFunction\n.extends fn<Int32>\n.end").is_err());
    let m = module("ldc.i4 42", "");
    let mut json = serde_json::to_value(m).unwrap();
    json["functions"][0]["owner"] = serde_json::to_value(parse_type("fn<Int32>").unwrap()).unwrap();
    json["functions"][0]["instance"] = true.into();
    json["functions"][0]["name"] = "$Function.Invoke".into();
    assert!(neoclr::load(&json.to_string()).is_err());
}

#[test]
fn repeated_bindings_compare_by_target_not_creation() {
    for (target, expected) in [
        ("First<Int32>()", true),
        ("First<String>()", false),
        ("Second()", false),
    ] {
        let m = assemble(&format!(".module Equality\n.entry Main\n.function First<T>() -> Int32\nldc.i4 42\nret\n.end\n.function Second() -> Int32\nldc.i4 42\nret\n.end\n.function Main() -> Boolean\nfunction.bind fn<Int32> = First<Int32>()\nfunction.bind fn<Int32> = {target}\nceq\nret\n.end")).unwrap();
        verify(&m).unwrap();
        assert_eq!(
            run(&m, Limits::default()).unwrap().value,
            Value::Boolean(expected)
        );
    }
}

#[test]
fn function_property_describes_bound_target_and_null_access_faults() {
    let declarations = ".function Identity<T>(T value) -> T\nldarg 0\nret\n.end";
    let source = format!(
        ".module Target\n.entry Main\n{declarations}\n.function Main() -> System.Introspection.MethodInfo\nfunction.bind fn<Int32,Int32> = Identity<Int32>(Int32)\ncall instance fn<Int32,Int32>::get_Function()\nret\n.end"
    );
    let m = assemble(&source).unwrap();
    verify(&m).unwrap();
    let result = run(&m, Limits::default()).unwrap();
    let Value::Object { fields, .. } = result.value else {
        panic!("expected method snapshot")
    };
    assert_eq!(fields[0], Value::String("Identity".into()));
    assert!(matches!(fields[1], Value::NullObjectReference(_)));
    let graph = LoadedProgram::new(&m)
        .unwrap()
        .analyze_reachability(
            &[neoclr::assembler::parse_function_ref("Main()").unwrap()],
            100,
        )
        .unwrap();
    assert!(
        graph.functions[graph.roots[0]]
            .function_invocations
            .is_empty()
    );
    let null = assemble(&source.replace(
        "function.bind fn<Int32,Int32> = Identity<Int32>(Int32)",
        ".local fn<Int32,Int32> empty\nldloca empty\ninitobj fn<Int32,Int32>\nldloc empty",
    ))
    .unwrap();
    verify(&null).unwrap();
    assert_eq!(
        run(&null, Limits::default()).unwrap_err().code,
        neoclr::FaultCode::NullReference
    );
}

#[test]
fn function_display_includes_closed_target_signature() {
    let source = ".module Display\n.entry Main\n.function Identity<T>(T) -> T\nldarg 0\nret\n.end\n.function Main() -> String\nfunction.bind fn<Int32,Int32> = Identity<Int32>(Int32)\ncallvirt instance fn<Int32,Int32>::ToString()\nret\n.end";
    let m = assemble(source).unwrap();
    verify(&m).unwrap();
    assert_eq!(
        run(&m, Limits::default()).unwrap().value,
        Value::String("[Display]Identity<System.Int32>(System.Int32) -> System.Int32".into())
    );
}

#[test]
fn function_object_roundtrip_preserves_value_and_reference_contracts() {
    let source = ".module ObjectFunctions\n.entry Main\n.type class abstract System.Object\n.end\n.function Keep<T>(T value) -> T\n.constraint T System.Object\nldarg value\nret\n.end\n.function Identity(Int32) -> Int32\nldarg 0\nret\n.end\n.function Main() -> Int32\n.local fn<Int32,Int32> first\n.local System.Object saved\nfunction.bind fn<Int32,Int32> = Identity(Int32)\ncall Keep<fn<Int32,Int32>>(fn<Int32,Int32>)\nstloc first\nldloc first\ncastclass System.Object\nstloc saved\nldloc saved\nisinst fn<Int32,Int32>\nref.isnull\nbrtrue Bad\nldloc saved\ncastclass fn<Int32,Int32>\nldloc first\nceq\nbrfalse Bad\nldloc saved\nldloc first\nref.eq\nbrfalse Bad\nldloc saved\nfunction.bind fn<Int32,Int32> = Identity(Int32)\nref.eq\nbrtrue Bad\nldloc saved\ncastclass fn<Int32,Int32>\nldc.i4 42\ncallvirt instance fn<Int32,Int32>::Invoke(Int32)\nret\nBad:\nldc.i4 0\nret\n.end";
    let m = assemble(source).unwrap();
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(42));
    let wrong = assemble(&source.replacen(
        "castclass fn<Int32,Int32>",
        "castclass fn<String,String>",
        1,
    ))
    .unwrap();
    assert_eq!(
        run(&wrong, Limits::default()).unwrap_err().code,
        neoclr::FaultCode::InvalidCast
    );
}

#[test]
fn generic_library_invokes_function_with_callers_internal_type() {
    let app = ".module App\n.references (Callbacks)\n.entry Main\n.type internal Payload\n.field Number Int32\n.end\n.function Read(Payload value) -> Int32\nldarg value\nldfld Payload::Number\nret\n.end\n.function Main() -> Int32\nfunction.bind fn<Payload,Int32> = Read(Payload)\nldc.i4 42\nnewobj Payload\ncall Apply<Payload>(fn<Payload,Int32>,Payload)\nret\n.end";
    let library = ".module Callbacks\n.function Apply<T>(fn<T,Int32> callback, T value) -> Int32\nldarg callback\nldarg value\ncallvirt instance fn<T,Int32>::Invoke(T)\nret\n.end";
    let modules = neoclr::assembler::assemble_modules(&[app, library]).unwrap();
    let program = LoadedProgram::with_modules(
        &modules[0],
        neoclr::library::system().unwrap(),
        &modules[1..],
    )
    .unwrap();
    program.verify().unwrap();
    assert_eq!(
        program
            .run(neoclr::ExecutionOptions::default())
            .unwrap()
            .value,
        Value::Int32(42)
    );

    // Explicitly naming the other module's internal type is still rejected.
    let illegal = library
        .replace(
            "Apply<T>(fn<T,Int32> callback, T value)",
            "Apply<T>(fn<Payload,Int32> callback, Payload value)",
        )
        .replace(
            "fn<T,Int32>::Invoke(T)",
            "fn<Payload,Int32>::Invoke(Payload)",
        );
    let error = neoclr::assembler::assemble_modules(&[app, &illegal])
        .and_then(|modules| {
            LoadedProgram::with_modules(
                &modules[0],
                neoclr::library::system().unwrap(),
                &modules[1..],
            )
            .map(|_| ())
        })
        .unwrap_err();
    assert!(error.message.contains("type access denied"), "{error}");
}
