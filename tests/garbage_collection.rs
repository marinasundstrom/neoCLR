use neoclr::{Limits, LoadedProgram, Value, assemble};

fn program(extra: &str, body: &str, returns: &str) -> LoadedProgram {
    let module = assemble(&format!(
        ".module App\n.entry Main\n{extra}\n.function Main() -> {returns}\n{body}\nret\n.end"
    ))
    .unwrap();
    let program = LoadedProgram::new(&module).unwrap();
    program.verify().unwrap();
    program
}

fn limits(objects: usize) -> Limits {
    Limits {
        heap_objects: objects,
        ..Limits::default()
    }
}

#[test]
fn allocation_budget_counts_live_objects_and_preserves_identities() {
    let body = format!(
        "{}ldc.i4 42\nheap.new",
        "ldc.i4 0\nheap.new\npop\n".repeat(100)
    );
    let result = program("", &body, "Ref<Int32>").run(limits(1)).unwrap();
    let Value::Reference { index, .. } = result.value else {
        panic!("expected reference")
    };
    assert_eq!(index, 100);
    assert_eq!(result.heap.get(index), Some(&Value::Int32(42)));
    assert!(result.heap.get(0).is_none());
    assert_eq!(result.heap.len(), 1);
    assert_eq!(result.heap.reclaimed_objects(), 100);
    let stats = result.heap.statistics();
    assert_eq!(stats.allocated_objects, 101);
    assert_eq!(stats.live_objects, 1);
    assert_eq!(stats.peak_objects, 1);
    assert_eq!(stats.collections, 101);
    assert_eq!(
        stats.allocated_objects,
        stats.live_objects + stats.reclaimed_objects
    );
}

#[test]
fn caller_cells_and_byref_targets_survive_callee_collection() {
    let extra = ".function Read(Ref<Int32>& item) -> Int32\nldc.i4 1\nheap.new\npop\nldc.i4 2\nheap.new\npop\nldarg item\nldobj Ref<Int32>\nheap.load\nret\n.end";
    let result = program(extra, ".local Ref<Int32> item\nldc.i4 42\nheap.new\nstloc item\nldloca item\ncall Read(Ref<Int32>&)", "Int32").run(limits(2)).unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert!(result.heap.is_empty());
    assert_eq!(result.heap.reclaimed_objects(), 3);
}

#[test]
fn pending_allocation_operand_is_a_root() {
    let fault = program("", "ldc.i4 42\nheap.new\nheap.new", "Ref<Ref<Int32>>")
        .run(limits(1))
        .unwrap_err();
    assert!(fault.message.contains("heap object limit"), "{fault:?}");
}

const CYCLE: &str = ".function Cycle() -> Ref<System.Value>\n.local Ref<System.Value> node\nldvoid\nvalue.pack Void\nheap.new\nstloc node\nldloc node\nldloc node\nvalue.pack Ref<System.Value>\nheap.store\npop\nldloc node\nret\n.end";

#[test]
fn unreachable_cycles_are_collected_under_allocation_pressure() {
    let result = program(
        CYCLE,
        "call Cycle()\npop\nldc.i4 42\nheap.new",
        "Ref<Int32>",
    )
    .run(limits(1))
    .unwrap();
    assert_eq!(result.heap.len(), 1);
    assert_eq!(result.heap.reclaimed_objects(), 1);
    assert_eq!(result.heap.get(1), Some(&Value::Int32(42)));
}

#[test]
fn returned_cycles_remain_reachable_without_recursive_marking() {
    let result = program(CYCLE, "call Cycle()", "Ref<System.Value>")
        .run(limits(1))
        .unwrap();
    assert_eq!(result.heap.len(), 1);
    assert_eq!(
        result.heap.get(0),
        Some(&Value::Erased(Box::new(result.value.clone())))
    );
    assert_eq!(result.heap.reclaimed_objects(), 0);
}

#[test]
fn inline_records_erasure_and_transitive_heap_edges_are_traced() {
    let extra = ".type Holder\n.field Item Ref<Int32>\n.end";
    let result = program(
        extra,
        "ldc.i4 42\nheap.new\nnewobj Holder\nvalue.pack Holder\nheap.new",
        "Ref<System.Value>",
    )
    .run(limits(2))
    .unwrap();
    assert_eq!(result.heap.len(), 2);
    assert_eq!(result.heap.get(0), Some(&Value::Int32(42)));
    assert_eq!(result.heap.reclaimed_objects(), 0);
}

#[test]
fn suspended_caller_evaluation_stack_is_a_root() {
    let extra = ".function Allocate() -> Void\nldc.i4 1\nheap.new\npop\nldc.i4 2\nheap.new\npop\nldvoid\nret\n.end";
    let result = program(
        extra,
        "ldc.i4 42\nheap.new\ncall Allocate()\npop\nheap.load",
        "Int32",
    )
    .run(limits(2))
    .unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert_eq!(result.heap.reclaimed_objects(), 3);
}
