//! Experimental native selection bounds, shared by lowering and code generation.
// HTTP + JSON + managed TLS requires more than the original small POC budget.
// Match NEOCLR_NATIVE_TYPE_LIMIT in the native host header.
pub const TYPES: usize = 1024;
// Match NEOCLR_NATIVE_FUNCTION_LIMIT in the native root-probe header.
// The source-built HTTP JSON sample currently selects 2,055 methods.
pub const FUNCTIONS: usize = 4096;
// Every generic clone also counts toward FUNCTIONS.
pub const FUNCTION_CLONES: usize = FUNCTIONS;
// The intermediate inventory contains source declarations plus specialized
// shapes. Selection still limits the executable image to TYPES. Raw linked
// source remains separately bounded by MAX_LOAD_SET_TYPES.
pub const INPUT_TYPES: usize = 2 * TYPES;
pub const INPUT_FUNCTIONS: usize = 8192;
// Match NEOCLR_NATIVE_LAYOUT_LANES in the native host header.
pub const INLINE_LANES: usize = 256;
// Heap objects use pointer ABI lanes regardless of payload size. Keep their
// allocation budget separate from inline values and record-array elements.
// Match NEOCLR_NATIVE_OBJECT_LANES in the native host header.
pub const OBJECT_LANES: usize = 256;

#[test]
fn type_budget_matches_native_collector_contract() {
    let header = include_str!("../../native/memory/text-arena.h");
    assert!(header.contains(&format!("#define NEOCLR_NATIVE_TYPE_LIMIT {TYPES}")));
}

#[test]
fn layout_budgets_match_native_allocator_contract() {
    let header = include_str!("../../native/memory/text-arena.h");
    assert!(header.contains(&format!(
        "#define NEOCLR_NATIVE_LAYOUT_LANES {INLINE_LANES}"
    )));
    assert!(header.contains(&format!(
        "#define NEOCLR_NATIVE_OBJECT_LANES {OBJECT_LANES}"
    )));
}

#[test]
fn function_budget_matches_native_root_probe_contract() {
    let header = include_str!("../../native/gc/root-probe.h");
    assert!(header.contains(&format!("#define NEOCLR_NATIVE_FUNCTION_LIMIT {FUNCTIONS}")));
}
