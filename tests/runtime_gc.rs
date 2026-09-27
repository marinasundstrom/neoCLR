use neoclr::{CollectionReason, Limits, LoadedProgram, Value, assemble};

const SERVICES: &str = r#"
.function neoCLR.Runtime.GCCollect() -> Void
.methodimpl InternalCall
.end
.function neoCLR.Runtime.GCCollectionCount() -> Int64
.methodimpl InternalCall
.end
.function neoCLR.Runtime.GCHeapObjectCount() -> Int64
.methodimpl InternalCall
.end
.function neoCLR.Runtime.GCAllocatedObjectCount() -> Int64
.methodimpl InternalCall
.end
.function neoCLR.Runtime.GCReclaimedObjectCount() -> Int64
.methodimpl InternalCall
.end
.function neoCLR.Runtime.GCPeakHeapObjectCount() -> Int64
.methodimpl InternalCall
.end
.function neoCLR.Runtime.GCHeapObjectLimit() -> Int64
.methodimpl InternalCall
.end
"#;
fn program(extra: &str, body: &str, returns: &str) -> LoadedProgram {
    let module = assemble(&format!(".module App\n.entry Main\n{SERVICES}{extra}\n.function Main() -> {returns}\n{body}\nret\n.end")).unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    program
}
#[test]
fn explicit_collection_reclaims_garbage_and_preserves_stack_roots() {
    let result = program("", "ldc.i4 42\nheap.new\nldc.i4 99\nheap.new\npop\ncall neoCLR.Runtime.GCCollect()\npop\nldobj Int32", "Int32").run(Limits::default()).unwrap();
    assert_eq!(result.value, Value::Int32(42));
    let event = result
        .heap
        .collection_events()
        .find(|event| event.reason == CollectionReason::ExplicitRequest)
        .unwrap();
    assert_eq!((event.before, event.after, event.reclaimed), (2, 1, 1));
    assert!(result.heap.is_empty());
}
#[test]
fn counters_are_execution_local_and_queries_do_not_collect() {
    let body = "ldc.i4 1\nheap.new\npop\ncall neoCLR.Runtime.GCAllocatedObjectCount()\ncall neoCLR.Runtime.GCHeapObjectCount()\nadd\ncall neoCLR.Runtime.GCPeakHeapObjectCount()\nadd\ncall neoCLR.Runtime.GCCollect()\npop\ncall neoCLR.Runtime.GCCollectionCount()\nadd\ncall neoCLR.Runtime.GCReclaimedObjectCount()\nadd\ncall neoCLR.Runtime.GCHeapObjectCount()\nadd";
    let p = program("", body, "Int64");
    for _ in 0..2 {
        let result = p.run(Limits::default()).unwrap();
        assert_eq!(result.value, Value::Int64(5));
        assert_eq!(result.heap.collections(), 2); // Explicit plus execution completion.
    }
    let result = program("", "call neoCLR.Runtime.GCHeapObjectLimit()", "Int64")
        .run(Limits {
            heap_objects: 7,
            ..Limits::default()
        })
        .unwrap();
    assert_eq!(result.value, Value::Int64(7));
}
#[test]
fn collection_preserves_caller_locals_and_interior_references() {
    let extra = ".function Read(Int32& value) -> Int32\ncall neoCLR.Runtime.GCCollect()\npop\nldarg value\nldobj Int32\nret\n.end";
    let result = program(
        extra,
        ".local Int32& value\nldc.i4 42\nheap.new\nstloc value\nldloc value\ncall Read(Int32&)",
        "Int32",
    )
    .run(Limits::default())
    .unwrap();
    assert_eq!(result.value, Value::Int32(42));
}
#[test]
fn explicit_collection_cannot_evade_heap_limit() {
    let result = program(
        "",
        "ldc.i4 1\nheap.new\ncall neoCLR.Runtime.GCCollect()\npop\nldc.i4 2\nheap.new\npop",
        "Int32&",
    )
    .run(Limits {
        heap_objects: 1,
        ..Limits::default()
    });
    assert_eq!(
        result.unwrap_err().code,
        neoclr::FaultCode::HeapLimitExceeded
    );
}
#[test]
fn service_signatures_are_exact() {
    for (name, args, result) in [
        ("GCCollect", "", "noresult"),
        ("GCCollect", "Int32", "noresult"),
        ("GCCollectionCount", "", "Int32"),
        ("GCHeapObjectLimit", "Int32", "Int64"),
        ("GCKeepAlive", "Int32", "noresult"),
        ("GCKeepAlive", "System.Object", "noresult"),
    ] {
        let module = assemble(&format!(
            ".module Bad\n.function neoCLR.Runtime.{name}({args}) -> {result}\n.methodimpl InternalCall\n.end"
        ));
        assert!(
            module
                .and_then(|m| LoadedProgram::new(&m))
                .and_then(|p| p.verify())
                .is_err(),
            "{name} {args} {result}"
        );
    }
}

#[test]
fn construction_and_keep_alive_use_the_same_heap_roots() {
    let extra = r#"
.type class abstract System.Object
.method instance .ctor() -> noresult
ret
.end
.end
.function neoCLR.Runtime.GCKeepAlive(System.Object value) -> Void
.methodimpl InternalCall
.end
.type class Item
.extends System.Object
.field Value Int32
.method instance .ctor() -> noresult
ldarg this
call instance System.Object::.ctor()
call neoCLR.Runtime.GCCollect()
pop
ldarg this
ldc.i4 42
stfld Item::Value
ret
.end
.end
"#;
    let result = program(extra, "newobj instance Item::.ctor()\ndup\ncastclass System.Object\ncall neoCLR.Runtime.GCKeepAlive(System.Object)\npop\nldfld Item::Value", "Int32").run(Limits::default()).unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert!(
        result
            .heap
            .collection_events()
            .any(|event| event.reason == CollectionReason::ExplicitRequest && event.after == 1)
    );
}

#[test]
fn explicit_collection_reclaims_unreachable_cycles() {
    let cycle = ".function Cycle() -> System.Value&\n.local System.Value& node\nldvoid\nvalue.pack Void\nheap.new\nstloc node\nldloc node\nldloc node\nvalue.pack System.Value&\nstobj System.Value\nldloc node\nret\n.end";
    let result = program(cycle, "call Cycle()\npop\ncall neoCLR.Runtime.GCCollect()\npop\ncall neoCLR.Runtime.GCHeapObjectCount()", "Int64").run(Limits::default()).unwrap();
    assert_eq!(result.value, Value::Int64(0));
    assert_eq!(result.heap.statistics().reclaimed_objects, 1);
}

#[test]
fn gc_control_reports_heap_requirement_and_consumes_instruction_budget() {
    let p = program(
        "",
        "again:\ncall neoCLR.Runtime.GCCollect()\npop\nbr again",
        "Int32",
    );
    let graph = p
        .analyze_reachability(
            &[neoclr::assembler::parse_function_ref("Main()").unwrap()],
            8,
        )
        .unwrap();
    assert!(
        graph
            .required_services()
            .contains(&neoclr::RuntimeService::ManagedHeap)
    );
    assert_eq!(
        p.run(Limits {
            instructions: 24,
            ..Limits::default()
        })
        .unwrap_err()
        .code,
        neoclr::FaultCode::InstructionLimitExceeded
    );
}
