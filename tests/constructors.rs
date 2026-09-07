use neoclr::{Limits, LoadedProgram, Value, assemble};

const SAMPLE: &str = include_str!("../examples/constructors.neoil");

fn source(body: &str, main: &str) -> String {
    format!(
        ".module App\n.entry Main\n.type Cell\n.field Stored Int32\n.method instance .ctor(Int32 value) -> Void\n{body}\n.end\n.end\n.function Main() -> Cell\n{main}\n.end"
    )
}

const INIT: &str = "ldarg value\nnewobj Cell\nstarg this\nldvoid\nret";
const MAIN: &str = "ldc.i4 42\nnewobj instance Cell::.ctor(Int32)\nret";

#[test]
fn overloads_bind_by_signature_and_reachability_follows_constructor_bodies() {
    let src = source(INIT, MAIN).replace("\n.end\n.end\n.function Main", "\n.end\n.method instance .ctor(String text) -> Void\nldarg text\ncall System.Console::WriteLine(String)\npop\nldc.i4 17\nnewobj Cell\nstarg this\nldvoid\nret\n.end\n.end\n.function Main");
    let src = src.replace(MAIN, "ldc.i4 42\nnewobj instance Cell::.ctor(Int32)\npop\nldstr \"string overload\"\nnewobj instance Cell::.ctor(String)\nret");
    let module = assemble(&src).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    let result = program.run(Limits::default()).unwrap();
    assert_eq!(result.output, ["string overload"]);
    let Value::Object { fields, .. } = result.value else {
        panic!("expected Cell")
    };
    assert_eq!(fields, [Value::Int32(17)]);
    let graph = program
        .analyze_reachability(
            &[neoclr::assembler::parse_function_ref("Main()").unwrap()],
            16,
        )
        .unwrap();
    let ctors: Vec<_> = graph
        .functions
        .iter()
        .filter(|f| f.target.name == "Cell..ctor")
        .collect();
    assert_eq!(ctors.len(), 2);
    assert_ne!(ctors[0].target.definition, ctors[1].target.definition);
    assert!(
        graph
            .functions
            .iter()
            .any(|f| f.target.name == "neoCLR.Runtime.WriteLine")
    );
}

#[test]
fn cross_module_construction_uses_public_constructor_over_private_fields() {
    let library = ".module Models\n.type Cell\n.field private Stored Int32\n.method public instance .ctor(Int32 value) -> Void\nldarg value\nnewobj Cell\nstarg this\nldvoid\nret\n.end\n.end";
    let app = format!(
        ".module App\n.references (Models)\n.entry Main\n.function Main() -> Cell\n{MAIN}\n.end"
    );
    let modules = neoclr::assembler::assemble_modules(&[&app, library]).unwrap();
    let program = LoadedProgram::with_modules(
        &modules[0],
        neoclr::library::system().unwrap(),
        &modules[1..],
    )
    .unwrap();
    program.verify().unwrap();
    assert!(program.run(Limits::default()).is_ok());
    assert!(
        neoclr::assembler::assemble_modules(&[
            &app,
            &library.replace(".method public", ".method internal")
        ])
        .is_err()
    );
}

#[test]
fn generic_private_constructors_round_trip_verify_and_execute() {
    let module = assemble(SAMPLE).unwrap();
    let json = serde_json::to_string(&module).unwrap();
    assert!(json.contains("newobj.ctor"));
    let module = neoclr::load(&json).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    assert_eq!(
        program.run(Limits::default()).unwrap().output,
        ["42", "Constructed through a public constructor"]
    );
}

#[test]
fn indexed_slots_and_explicit_opcode_match_named_syntax() {
    for (body, main) in [
        (INIT.to_owned(), MAIN.to_owned()),
        (
            INIT.replace("value", "1").replace("this", "0"),
            MAIN.replace("newobj instance", "newobj.ctor instance"),
        ),
    ] {
        let module = assemble(&source(&body, &main)).unwrap();
        let program = LoadedProgram::new(&module).unwrap();
        program.verify().unwrap();
        let Value::Object { fields, .. } = program.run(Limits::default()).unwrap().value else {
            panic!("expected Cell")
        };
        assert_eq!(fields, [Value::Int32(42)]);
    }
}

#[test]
fn uninitialized_receiver_faults_with_constructor_and_caller_frames() {
    for body in ["ldarg this\npop\nldvoid\nret", "ldvoid\nret"] {
        let module = assemble(&source(body, MAIN)).unwrap();
        let program = LoadedProgram::new(&module).unwrap();
        assert!(
            program
                .verify()
                .unwrap_err()
                .message
                .contains("not initialized")
        );
        let fault = program.run(Limits::default()).unwrap_err();
        assert!(fault.message.contains("initializ"), "{fault}");
        let trace = fault.stack_trace.unwrap();
        assert_eq!(trace.frames[0].function.name, "Cell..ctor");
        assert_eq!(trace.frames[1].function.name, "Main");
    }
}

#[test]
fn initialization_must_hold_on_every_return_path() {
    let body =
        "ldarg value\nbrfalse done\nldarg value\nnewobj Cell\nstarg this\ndone:\nldvoid\nret";
    let module = assemble(&source(body, MAIN)).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    assert!(program.verify().is_err());
    assert!(program.run(Limits::default()).is_ok());
    let module = assemble(&source(body, &MAIN.replace("42", "0"))).unwrap();
    assert!(
        LoadedProgram::new(&module)
            .unwrap()
            .run(Limits::default())
            .is_err()
    );
}

#[test]
fn receiver_storage_and_void_return_remain_type_checked() {
    for body in [
        "ldc.i4 7\nstarg this\nldvoid\nret",
        "ldarg value\nnewobj Cell\nstarg this\nldc.i4 7\nret",
        "ldarg value\nnewobj Cell\nstarg this\nldvoid\nldvoid\nret",
    ] {
        let module = assemble(&source(body, MAIN)).unwrap();
        let program = LoadedProgram::new(&module).unwrap();
        assert!(program.verify().is_err());
        assert!(program.run(Limits::default()).is_err());
    }
}

#[test]
fn empty_records_have_a_complete_receiver_without_a_store() {
    let src = ".module Empty\n.entry Main\n.type Unit\n.method instance .ctor() -> Void\nldarg this\npop\nldvoid\nret\n.end\n.end\n.function Main() -> Unit\nnewobj instance Unit::.ctor()\nret\n.end";
    let module = assemble(src).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    let Value::Object { fields, .. } = program.run(Limits::default()).unwrap().value else {
        panic!("expected Unit")
    };
    assert!(fields.is_empty());
}

#[test]
fn construction_enforces_access_target_kind_and_frame_limits() {
    let src = source(INIT, MAIN);
    assert!(assemble(&src.replace(".method instance", ".method private instance")).is_err());
    assert!(assemble(&src.replace(".ctor", "Create")).is_err());
    assert!(assemble(&src.replace("instance", "static")).is_err());
    let module = assemble(&src).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    let fault = program
        .run(Limits {
            frames: 1,
            ..Limits::default()
        })
        .unwrap_err();
    assert!(fault.message.contains("frame limit"));
}

#[test]
fn ordinary_constructor_calls_keep_value_receiver_semantics() {
    let main = "ldc.i4 9\nnewobj Cell\ndup\nldc.i4 42\ncall instance Cell::.ctor(Int32)\npop\nret";
    let module = assemble(&source(INIT, main)).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    let Value::Object { fields, .. } = program.run(Limits::default()).unwrap().value else {
        panic!("expected Cell")
    };
    assert_eq!(fields, [Value::Int32(9)]);
}
