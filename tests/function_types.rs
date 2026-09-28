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
